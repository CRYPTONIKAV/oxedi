//! The viewer's picture of one file: segments with their byte ranges and
//! owning loop instance, the loop tree, the projected tables as text and
//! the diagnostics with their location.

use std::sync::OnceLock;

use oxedi_core::{Cell, Diagnostic, Document, LoopTree, NodeId, Processor, Spec, Tables};
use serde::Serialize;

#[cfg(test)]
mod tests;

static BUILTIN: OnceLock<Spec> = OnceLock::new();
static BUILTIN_4010: OnceLock<Spec> = OnceLock::new();

fn builtin() -> &'static Spec {
    BUILTIN.get_or_init(Spec::builtin_835)
}

fn builtin_4010() -> &'static Spec {
    BUILTIN_4010.get_or_init(Spec::builtin_835_4010)
}

/// The built-in spec of the version `document` declares, else the default.
pub(crate) fn select(document: &Document<'_>) -> &'static Spec {
    Spec::select(&[builtin(), builtin_4010()], builtin(), document.segments())
}

/// One file as the viewer shows it.
#[derive(Debug, Serialize)]
pub struct View {
    /// The declared version the chosen spec covers, e.g. `005010X221A1`.
    pub spec_version: Option<String>,
    /// The file's separators, as text.
    pub delimiters: Delimiters,
    /// Every segment, in file order.
    pub segments: Vec<SegmentView>,
    /// Loop instances; index 0 is the root.
    pub nodes: Vec<NodeView>,
    /// The spec's tables, ordered by name.
    pub tables: Vec<TableView>,
    /// Every finding, in the order the core reports them.
    pub diagnostics: Vec<DiagnosticView>,
}

/// The separators of a file.
#[derive(Debug, Serialize)]
pub struct Delimiters {
    pub element: String,
    pub component: String,
    pub segment: String,
    pub repetition: Option<String>,
}

/// One segment: its id, where its raw bytes live and the loop instance that holds it.
#[derive(Debug, Serialize)]
pub struct SegmentView {
    pub id: String,
    /// First byte of the segment's raw range (trivia, body, terminator).
    pub start: usize,
    /// One past its last byte.
    pub end: usize,
    /// Index into `View::nodes`.
    pub node: usize,
}

/// One loop instance.
#[derive(Debug, Serialize)]
pub struct NodeView {
    /// The loop's name as the spec writes it; `None` for the root.
    pub loop_name: Option<String>,
    /// 1 for the first instance of its loop in the file, 2 for the second…; 0 for the root.
    pub ordinal: usize,
    /// `true` when no segment of its own opened it.
    pub implicit: bool,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

/// One projected table, column by column.
#[derive(Debug, Serialize)]
pub struct TableView {
    pub name: String,
    pub rows: usize,
    pub columns: Vec<ColumnView>,
}

/// One column: each row as text, `None` for a null.
#[derive(Debug, Serialize)]
pub struct ColumnView {
    pub name: String,
    pub values: Vec<Option<String>>,
}

/// One finding with its full text and where it points.
#[derive(Debug, Serialize)]
pub struct DiagnosticView {
    pub text: String,
    pub level: String,
    pub segment: Option<usize>,
    pub element: Option<usize>,
    pub component: Option<usize>,
    /// Open loops, outermost first, as `(name, ordinal)`.
    pub path: Vec<(String, usize)>,
}

/// Parses `bytes` with the built-in spec of their declared version; the
/// error is the core's message for bytes that are not an X12 interchange.
pub fn view(bytes: &[u8]) -> Result<View, String> {
    let document = Document::parse(bytes).map_err(|err| err.to_string())?;
    let spec = select(&document);
    let (tables, diagnostics) = Processor::run(spec, &document);
    let tree = LoopTree::build(spec, document.segments());
    let delimiters = document.delimiters();
    let text = |byte: u8| char::from(byte).to_string();
    Ok(View {
        spec_version: spec
            .version()
            .and_then(|version| version.values.first())
            .map(|value| String::from_utf8_lossy(value).into_owned()),
        delimiters: Delimiters {
            element: text(delimiters.element),
            component: text(delimiters.component),
            segment: text(delimiters.segment),
            repetition: delimiters.repetition.map(text),
        },
        segments: segments(&document, &tree),
        nodes: nodes(spec, &tree),
        tables: tables_view(&tables),
        diagnostics: diagnostics.iter().map(diagnostic).collect(),
    })
}

/// [`view`] as the JSON the page reads.
pub fn to_json(bytes: &[u8]) -> Result<String, String> {
    serde_json::to_string(&view(bytes)?).map_err(|err| err.to_string())
}

fn segments(document: &Document<'_>, tree: &LoopTree) -> Vec<SegmentView> {
    let mut owner = vec![0usize; document.len()];
    for (index, node) in tree.nodes().iter().enumerate() {
        for &segment in node.segments.iter().chain(&node.unmatched) {
            if let Some(slot) = owner.get_mut(segment) {
                *slot = index;
            }
        }
    }
    document
        .spans()
        .zip(owner)
        .enumerate()
        .map(|(index, (span, node))| SegmentView {
            id: String::from_utf8_lossy(document.segment_id(index).unwrap_or_default())
                .into_owned(),
            start: span.raw.start,
            end: span.raw.end,
            node,
        })
        .collect()
}

fn nodes(spec: &Spec, tree: &LoopTree) -> Vec<NodeView> {
    let mut seen = vec![0usize; spec.loops().len()];
    tree.nodes()
        .iter()
        .map(|node| {
            let (loop_name, ordinal) = match node.loop_id {
                Some(id) => {
                    let ordinal = seen.get_mut(id.index()).map_or(0, |count| {
                        *count = count.saturating_add(1);
                        *count
                    });
                    (Some(spec.loop_name(id).to_string()), ordinal)
                }
                None => (None, 0),
            };
            NodeView {
                loop_name,
                ordinal,
                implicit: node.implicit,
                parent: node.parent.map(NodeId::index),
                children: node.children.iter().map(|child| child.index()).collect(),
            }
        })
        .collect()
}

fn tables_view(tables: &Tables) -> Vec<TableView> {
    tables
        .iter()
        .map(|table| TableView {
            name: table.name().to_string(),
            rows: table.len(),
            columns: table
                .columns()
                .iter()
                .map(|(name, data)| ColumnView {
                    name: name.clone(),
                    values: (0..data.len())
                        .map(|row| match data.get(row) {
                            Some(Cell::Null) | None => None,
                            Some(_) => data.render(row),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

fn diagnostic(diagnostic: &Diagnostic) -> DiagnosticView {
    DiagnosticView {
        text: diagnostic.to_string(),
        level: diagnostic.level.to_string(),
        segment: diagnostic.segment,
        element: diagnostic.element,
        component: diagnostic.component,
        path: diagnostic
            .path
            .iter()
            .map(|loop_ref| (loop_ref.name.clone(), loop_ref.ordinal))
            .collect(),
    }
}
