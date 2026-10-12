use super::*;
use oxedi_core::{Document, Processor, Spec};
use std::path::{Path, PathBuf};

fn corpus() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../oxedi_core/tests");
    let mut files = Vec::new();
    for dir in ["fixtures", "samples"] {
        for entry in std::fs::read_dir(root.join(dir)).expect("test corpus") {
            let path = entry.expect("entry").path();
            if path.file_name().is_some_and(|n| n != "README.md") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn parsable(path: &Path) -> Option<Vec<u8>> {
    let bytes = std::fs::read(path).expect("read");
    let parses = Document::parse(&bytes[..]).is_ok();
    parses.then_some(bytes)
}

#[test]
fn ranges_are_contiguous_and_cover_the_file() {
    for path in corpus() {
        let Some(bytes) = parsable(&path) else {
            continue;
        };
        let view = view(&bytes).expect("view");
        let mut at = 0;
        for segment in &view.segments {
            assert_eq!(segment.start, at, "{path:?}");
            assert!(segment.end > segment.start, "{path:?}");
            at = segment.end;
        }
        assert_eq!(at, bytes.len(), "{path:?}");
    }
}

#[test]
fn every_segment_with_content_belongs_to_one_node() {
    for path in corpus() {
        let Some(bytes) = parsable(&path) else {
            continue;
        };
        let view = view(&bytes).expect("view");
        let mut owners = vec![0usize; view.segments.len()];
        let document = Document::parse(&bytes[..]).expect("parse");
        let spec = select(&document);
        let tree = oxedi_core::LoopTree::build(spec, document.segments());
        for node in tree.nodes() {
            for &index in node.segments.iter().chain(&node.unmatched) {
                owners[index] += 1;
            }
        }
        for (index, &count) in owners.iter().enumerate() {
            // A segment with no content (trailing trivia, `~~`) has no loop;
            // the view gives it to the root.
            let empty = document.span(index).expect("span").body.is_empty();
            assert_eq!(count, usize::from(!empty), "{path:?} segment {index}");
            if empty {
                assert_eq!(view.segments[index].node, 0, "{path:?} segment {index}");
            }
        }
        assert!(
            view.segments.iter().all(|s| s.node < view.nodes.len()),
            "{path:?}"
        );
    }
}

#[test]
fn tables_and_diagnostics_match_the_core() {
    for path in corpus() {
        let Some(bytes) = parsable(&path) else {
            continue;
        };
        let view = view(&bytes).expect("view");
        let document = Document::parse(&bytes[..]).expect("parse");
        let (tables, diagnostics) = Processor::run(select(&document), &document);
        assert_eq!(view.tables.len(), tables.len(), "{path:?}");
        for (mine, theirs) in view.tables.iter().zip(tables.iter()) {
            assert_eq!(mine.name, theirs.name());
            assert_eq!(mine.rows, theirs.len());
            assert_eq!(mine.columns.len(), theirs.columns().len());
        }
        let texts: Vec<String> = diagnostics.iter().map(ToString::to_string).collect();
        let mine: Vec<&str> = view.diagnostics.iter().map(|d| d.text.as_str()).collect();
        assert_eq!(mine, texts, "{path:?}");
    }
}

#[test]
fn a_4010_file_uses_the_4010_spec() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../oxedi_core/tests/fixtures/balanced_4010_sample.txt");
    let view = view(&std::fs::read(path).expect("read")).expect("view");
    let expected = Spec::builtin_835_4010();
    let declared = expected
        .version()
        .and_then(|v| v.values.first())
        .expect("declared");
    assert_eq!(
        view.spec_version.as_deref(),
        Some(&*String::from_utf8_lossy(declared))
    );
    assert!(view.diagnostics.is_empty(), "{:?}", view.diagnostics);
}

#[test]
fn a_null_cell_is_none_and_a_value_is_its_text() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../oxedi_core/tests/fixtures/balanced_5010_sample.txt");
    let view = view(&std::fs::read(path).expect("read")).expect("view");
    let payments = view
        .tables
        .iter()
        .find(|t| t.name == "payments")
        .expect("payments");
    let total = payments
        .columns
        .iter()
        .find(|c| c.name == "total_payment_amount")
        .expect("col");
    assert!(total.values[0].as_deref().is_some_and(|v| v.contains('.')));
    assert!(
        payments
            .columns
            .iter()
            .any(|c| c.values.iter().any(Option::is_none))
    );
}

#[test]
fn bytes_that_are_not_an_835_give_the_core_error() {
    for bytes in [
        &b""[..],
        &b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n"[..],
        &b"a,b,c\n1,2,3\n"[..],
    ] {
        let expected = Document::parse(bytes).expect_err("not an 835").to_string();
        assert_eq!(view(bytes).expect_err("error"), expected);
    }
}

#[test]
fn the_view_serializes_with_the_documented_keys() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../oxedi_core/tests/fixtures/balanced_5010_sample.txt");
    let json = to_json(&std::fs::read(path).expect("read")).expect("json");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    for key in [
        "spec_version",
        "delimiters",
        "segments",
        "nodes",
        "tables",
        "diagnostics",
    ] {
        assert!(value.get(key).is_some(), "{key}");
    }
    assert_eq!(value["nodes"][0]["loop_name"], serde_json::Value::Null);
    assert_eq!(value["delimiters"]["element"], "*");
}
