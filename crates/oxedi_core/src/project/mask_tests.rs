use super::*;
use crate::{DateMode, Document, Mask, Phi, Rule, SnipLevel};

const SAMPLE: &[u8] = include_bytes!("../../tests/fixtures/balanced_5010_sample.txt");
const KEY: &[u8] = b"oxedi-test-key-0";

fn mask() -> Mask {
    Mask::new(KEY, DateMode::Year).unwrap()
}

/// The index of the first segment `id` whose first element is `first`, and
/// that element.
fn find(document: &Document<'_>, id: &[u8], first: Option<&[u8]>) -> (usize, Vec<u8>) {
    document
        .segments()
        .find_map(|segment| {
            let value = segment.element(1)?.simple()?.to_vec();
            (segment.id == id && first.is_none_or(|first| first == value.as_slice()))
                .then_some((segment.index, value))
        })
        .unwrap()
}

fn external(message: String, segment: Option<usize>, element: usize, datum: &[u8]) -> Diagnostic {
    Diagnostic::new(
        Rule::External {
            origin: "pyx12".into(),
            code: Some("8".into()),
            message,
            level: SnipLevel::L2,
        },
        segment,
        Some(element),
        None,
        vec![],
        datum.to_vec(),
    )
}

fn message(diagnostic: &Diagnostic) -> &str {
    match &diagnostic.rule {
        Rule::External { message, .. } => message,
        _ => panic!("not an external finding"),
    }
}

#[test]
fn mask_found_masks_the_datum_and_message_of_a_finding_on_a_flagged_place() {
    let spec = Spec::builtin_835();
    let document = Document::parse(SAMPLE).unwrap();
    let mask = mask();
    let projector = Projector::new(&spec, document.delimiters()).with_mask(&mask);
    let (index, id) = find(&document, b"CLP", None);
    let text = String::from_utf8(id.clone()).unwrap();
    let mut found = vec![external(
        format!("Invalid value '{text}' in CLP01"),
        Some(index),
        1,
        &id,
    )];

    projector.mask_found(&mut found, &document);

    let token = mask.text(Phi::Id, &id);
    assert_eq!(found[0].datum, token.to_vec());
    let token = String::from_utf8_lossy(&token).into_owned();
    assert!(!message(&found[0]).contains(&text));
    assert_eq!(
        message(&found[0]),
        format!("Invalid value '{token}' in CLP01")
    );
}

#[test]
fn mask_found_leaves_a_finding_on_an_unflagged_place_as_written() {
    let spec = Spec::builtin_835();
    let document = Document::parse(SAMPLE).unwrap();
    let mask = mask();
    let projector = Projector::new(&spec, document.delimiters()).with_mask(&mask);
    let (index, _) = find(&document, b"N1", Some(b"PR"));
    let name = b"ACME HEALTH PLAN";
    let original = external(
        "Invalid value 'ACME HEALTH PLAN' in N102".into(),
        Some(index),
        2,
        name,
    );
    let mut found = vec![original.clone()];

    projector.mask_found(&mut found, &document);

    assert_eq!(found, vec![original]);
}

#[test]
fn mask_found_leaves_a_finding_without_a_segment_or_past_the_end_as_written() {
    let spec = Spec::builtin_835();
    let document = Document::parse(SAMPLE).unwrap();
    let mask = mask();
    let projector = Projector::new(&spec, document.delimiters()).with_mask(&mask);
    let (_, id) = find(&document, b"CLP", None);
    let text = String::from_utf8(id.clone()).unwrap();
    let originals = vec![
        external(format!("Invalid value '{text}'"), None, 1, &id),
        external(format!("Invalid value '{text}'"), Some(usize::MAX), 1, &id),
    ];
    let mut found = originals.clone();

    projector.mask_found(&mut found, &document);

    assert_eq!(found, originals);
}

#[test]
fn mask_found_does_nothing_without_a_mask() {
    let spec = Spec::builtin_835();
    let document = Document::parse(SAMPLE).unwrap();
    let projector = Projector::new(&spec, document.delimiters());
    let (index, id) = find(&document, b"CLP", None);
    let text = String::from_utf8(id.clone()).unwrap();
    let original = external(format!("Invalid value '{text}'"), Some(index), 1, &id);
    let mut found = vec![original.clone()];

    projector.mask_found(&mut found, &document);

    assert_eq!(found, vec![original]);
}

#[test]
fn mask_found_replaces_every_occurrence_of_a_one_character_datum() {
    let spec = Spec::builtin_835();
    let document = Document::parse(SAMPLE).unwrap();
    let mask = mask();
    let projector = Projector::new(&spec, document.delimiters()).with_mask(&mask);
    let (index, _) = find(&document, b"CLP", None);
    let mut found = vec![external("1 of 1".into(), Some(index), 1, b"1")];

    projector.mask_found(&mut found, &document);

    let token = mask.text(Phi::Id, b"1");
    assert_eq!(found[0].datum, token.to_vec());
    let token = String::from_utf8_lossy(&token).into_owned();
    assert_eq!(message(&found[0]), format!("{token} of {token}"));
}

#[test]
fn mask_found_replaces_a_datum_quoted_as_latin_1_text() {
    let spec = Spec::builtin_835();
    let document = Document::parse(SAMPLE).unwrap();
    let mask = mask();
    let projector = Projector::new(&spec, document.delimiters()).with_mask(&mask);
    let (index, _) = find(&document, b"NM1", Some(b"QC"));
    let datum = b"D\xe9OE";
    let mut found = vec![external(
        "contains an invalid character(D\u{e9}OE)".into(),
        Some(index),
        3,
        datum,
    )];

    projector.mask_found(&mut found, &document);

    let token = mask.text(Phi::Name, datum);
    assert_eq!(found[0].datum, token.to_vec());
    let token = String::from_utf8_lossy(&token).into_owned();
    assert_eq!(
        message(&found[0]),
        format!("contains an invalid character({token})")
    );
}

#[test]
fn mask_found_masks_flagged_values_a_segment_level_message_quotes() {
    let spec = Spec::builtin_835();
    let document = Document::parse(SAMPLE).unwrap();
    let mask = mask();
    let projector = Projector::new(&spec, document.delimiters()).with_mask(&mask);
    let (index, id) = find(&document, b"CLP", None);
    let text = String::from_utf8(id.clone()).unwrap();
    let mut found = vec![Diagnostic::new(
        Rule::External {
            origin: "pyx12".into(),
            code: Some("1".into()),
            message: format!("Segment CLP*{text} not found."),
            level: SnipLevel::L2,
        },
        Some(index),
        None,
        None,
        vec![],
        b"CLP".to_vec(),
    )];

    projector.mask_found(&mut found, &document);

    let token = String::from_utf8_lossy(&mask.text(Phi::Id, &id)).into_owned();
    assert_eq!(found[0].datum, b"CLP".to_vec());
    assert_eq!(
        message(&found[0]),
        format!("Segment CLP*{token} not found.")
    );
}

#[test]
fn mask_found_rewrites_a_message_once_when_the_datum_and_the_segment_quote_values() {
    let spec = Spec::builtin_835();
    let document = Document::parse(SAMPLE).unwrap();
    let mask = mask();
    let projector = Projector::new(&spec, document.delimiters()).with_mask(&mask);
    let (index, id) = find(&document, b"CLP", None);
    let text = String::from_utf8(id.clone()).unwrap();
    let mut found = vec![external(
        format!("1 is not CLP*{text}"),
        Some(index),
        1,
        b"1",
    )];

    projector.mask_found(&mut found, &document);

    let one = String::from_utf8_lossy(&mask.text(Phi::Id, b"1")).into_owned();
    let token = String::from_utf8_lossy(&mask.text(Phi::Id, &id)).into_owned();
    assert_eq!(message(&found[0]), format!("{one} is not CLP*{token}"));
}

#[test]
fn mask_found_leaves_values_of_another_segment_in_the_message_as_written() {
    let spec = Spec::builtin_835();
    let document = Document::parse(SAMPLE).unwrap();
    let mask = mask();
    let projector = Projector::new(&spec, document.delimiters()).with_mask(&mask);
    let (_, id) = find(&document, b"CLP", None);
    let (index, _) = find(&document, b"TRN", None);
    let text = String::from_utf8(id).unwrap();
    let original = Diagnostic::new(
        Rule::External {
            origin: "pyx12".into(),
            code: Some("1".into()),
            message: format!("Segment CLP*{text} not found."),
            level: SnipLevel::L2,
        },
        Some(index),
        None,
        None,
        vec![],
        b"TRN".to_vec(),
    );
    let mut found = vec![original.clone()];

    projector.mask_found(&mut found, &document);

    assert_eq!(found, vec![original]);
}
