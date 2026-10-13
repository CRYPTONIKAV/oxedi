//! Masked projections of every fixture and sample, each under the built-in
//! spec it selects: the same tables, columns and rows as without a mask, the
//! columns the spec does not flag unchanged, the flagged ones deterministic
//! per key, and no original value of a flagged column left in any cell or
//! diagnostic text.

mod common;

use oxedi_core::{Cell, DateMode, Diagnostic, Document, Mask, Phi, Processor, Spec, Table, Tables};

const KEY: &[u8] = b"oxedi-test-key-0";
const OTHER_KEY: &[u8] = b"oxedi-test-key-1";

/// Every file with its bytes, its spec, and its unmasked and masked runs.
fn runs(
    dates: DateMode,
    mut check: impl FnMut(&str, &Spec, (&Tables, &[Diagnostic]), (&Tables, &[Diagnostic])),
) {
    let (five, four) = common::builtins();
    let mask = Mask::new(KEY, dates).unwrap();
    for (name, bytes, delims) in common::all_files() {
        let spec = common::select(&five, &four, &bytes, delims);
        let document = Document::with_delimiters(&bytes[..], delims).unwrap();
        let (plain, plain_diagnostics) = Processor::run(spec, &document);
        let (masked, masked_diagnostics) = Processor::run_masked(spec, &document, &mask);
        check(
            &name,
            spec,
            (&plain, &plain_diagnostics),
            (&masked, &masked_diagnostics),
        );
    }
}

/// The kind of patient data each column of `table` holds, by column name.
fn flag(spec: &Spec, table: &Table, column: &str) -> Option<Phi> {
    let def = spec.tables().iter().find(|def| def.name == table.name())?;
    let index = def.columns.iter().position(|(name, _)| name == column)?;
    def.phi(index)
}

#[test]
fn a_mask_keeps_the_shape_and_every_column_it_does_not_flag() {
    runs(
        DateMode::Year,
        |name, spec, (plain, plain_d), (masked, masked_d)| {
            assert_eq!(plain.len(), masked.len(), "{name}");
            for (before, after) in plain.iter().zip(masked) {
                assert_eq!(before.name(), after.name(), "{name}");
                assert_eq!(before.len(), after.len(), "{name} {}", before.name());
                let kinds = |table: &Table| -> Vec<String> {
                    table
                        .columns()
                        .iter()
                        .map(|(column, data)| format!("{column}: {}", data.kind()))
                        .collect()
                };
                assert_eq!(kinds(before), kinds(after), "{name} {}", before.name());
                for ((column, was), (_, now)) in before.columns().iter().zip(after.columns()) {
                    for row in 0..before.len() {
                        let (was, now) = (was.get(row).unwrap(), now.get(row).unwrap());
                        let at = format!("{name} {}.{column} row {row}", before.name());
                        match (flag(spec, before, column), was) {
                            (None, _) => assert_eq!(was, now, "{at}"),
                            (Some(_), Cell::Null) => assert_eq!(now, Cell::Null, "{at}"),
                            (Some(_), Cell::Binary(b"")) => {
                                assert_eq!(now, Cell::Binary(b""), "{at}")
                            }
                            (Some(Phi::Name | Phi::Id), Cell::Binary(_)) => {
                                let Cell::Binary(token) = now else {
                                    panic!("{at}: {now:?}")
                                };
                                assert_eq!(token.len(), 16, "{at}");
                                assert!(
                                    token
                                        .iter()
                                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
                                    "{at}"
                                );
                            }
                            (Some(Phi::Date), Cell::Date32(days)) => {
                                assert_eq!(now, Cell::Date32(Mask::year_start(days)), "{at}");
                            }
                            (Some(kind), other) => panic!("{at}: {kind:?} column holds {other:?}"),
                        }
                    }
                }
            }
            assert_eq!(plain_d.len(), masked_d.len(), "{name}");
            for (before, after) in plain_d.iter().zip(masked_d) {
                assert_eq!(
                    (
                        &before.rule,
                        before.segment,
                        before.element,
                        before.component,
                        &before.path
                    ),
                    (
                        &after.rule,
                        after.segment,
                        after.element,
                        after.component,
                        &after.path
                    ),
                    "{name}: only the datum may change"
                );
            }
        },
    );
}

#[test]
fn masked_cells_depend_on_the_key_only() {
    let (five, four) = common::builtins();
    let mask = Mask::new(KEY, DateMode::Shift).unwrap();
    let other = Mask::new(OTHER_KEY, DateMode::Shift).unwrap();
    for (name, bytes, delims) in common::all_files() {
        let spec = common::select(&five, &four, &bytes, delims);
        let document = Document::with_delimiters(&bytes[..], delims).unwrap();
        let once = Processor::run_masked(spec, &document, &mask);
        let again = Processor::run_masked(spec, &document, &mask);
        assert_eq!(once, again, "{name}: the same key gives the same output");
        let (elsewhere, _) = Processor::run_masked(spec, &document, &other);
        for (table, changed) in once.0.iter().zip(&elsewhere) {
            for ((column, a), (_, b)) in table.columns().iter().zip(changed.columns()) {
                if !matches!(flag(spec, table, column), Some(Phi::Name | Phi::Id)) {
                    continue;
                }
                for row in 0..table.len() {
                    if let Some(Cell::Binary(value)) = a.get(row)
                        && !value.is_empty()
                    {
                        assert_ne!(
                            a.get(row),
                            b.get(row),
                            "{name} {}.{column} row {row}: another key gives another token",
                            table.name()
                        );
                    }
                }
            }
        }
    }
}

/// Every non-empty value of a flagged column, as the file writes it: text as
/// is, dates as `CCYYMMDD`, and dates also as tables render them unless
/// masking leaves them unchanged (already January 1).
fn originals(spec: &Spec, plain: &Tables) -> Vec<(Phi, String)> {
    let mut found = Vec::new();
    for table in plain {
        for (column, data) in table.columns() {
            let Some(kind) = flag(spec, table, column) else {
                continue;
            };
            for row in 0..table.len() {
                match data.get(row) {
                    Some(Cell::Binary(value)) if !value.is_empty() => {
                        found.push((kind, String::from_utf8_lossy(value).into_owned()));
                    }
                    Some(Cell::Date32(days)) => {
                        let rendered = data.render(row).unwrap();
                        found.push((kind, rendered.replace('-', "")));
                        if Mask::year_start(days) != days {
                            found.push((kind, rendered));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    found.sort_by(|a, b| a.1.cmp(&b.1));
    found.dedup();
    found
}

/// Whether `text` is a token: 16 lowercase hex characters.
fn is_token(text: &str) -> bool {
    text.len() == 16
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A masked cell leaks an original of its own kind when it holds it; a
/// token is hex, so a short original made of hex digits can appear in one by
/// chance, and a token leaks only by being the original. A diagnostic's text
/// also holds positions and counts, so an original shorter than six bytes
/// leaks there only as a whole quoted datum.
///
/// Every cell of every table is scanned, flagged or not, for an `id`
/// original of six or more characters (a patient control number can
/// ride in an unflagged column, such as a provider adjustment reference): no
/// cell contains it, except that a token leaks only by being the original
/// and an unflagged cell holds a shorter original only as the whole cell (an
/// anonymized sample has a nine-digit patient id that is a prefix of a
/// ten-digit provider identifier).
/// A `name` and a shorter `id` original are checked against flagged cells
/// only: a surname is short and common, and provider data can equal it by
/// coincidence (a rendering provider named like a patient), which is not a
/// leak.
#[test]
fn no_original_value_of_a_flagged_column_survives_masking() {
    for dates in [DateMode::Year, DateMode::Shift] {
        no_original_survives(dates);
    }
}

fn no_original_survives(dates: DateMode) {
    runs(dates, |name, spec, (plain, _), (masked, diagnostics)| {
        let originals = originals(spec, plain);
        assert!(!originals.is_empty(), "{name} has patient data to mask");
        let mut cells = Vec::new();
        for table in masked {
            for (column, data) in table.columns() {
                let kind = flag(spec, table, column);
                cells.extend((0..table.len()).filter_map(|row| Some((kind, data.render(row)?))));
            }
        }
        let texts: Vec<String> = diagnostics.iter().map(ToString::to_string).collect();
        for (kind, original) in &originals {
            let is_date = *kind == Phi::Date;
            // A shifted date can land on the original date of another row, so
            // dates are compared in cells only when they reduce to the year.
            let in_cells = !(is_date && dates == DateMode::Shift);
            for (own, cell) in cells.iter().filter(|(own, _)| {
                in_cells
                    && match own {
                        Some(own) => (*own == Phi::Date) == is_date,
                        None => *kind == Phi::Id && original.len() >= 6,
                    }
            }) {
                let leaks = if is_token(cell) || (own.is_none() && original.len() < 10) {
                    cell == original
                } else {
                    cell.contains(original.as_str())
                };
                assert!(!leaks, "{name}: {original:?} leaks in cell {cell:?}");
            }
            let quoted = format!("{original:?}");
            for text in &texts {
                let leaks = text.contains(&quoted)
                    || (original.len() >= 6 && text.contains(original.as_str()));
                assert!(!leaks, "{name}: {original:?} leaks in {text:?}");
            }
        }
    });
}

/// The fixtures built to carry findings on flagged places: one finding each on
/// a claim id, a member id, a group number, a statement date and a line item
/// control number, under both versions.
#[test]
fn a_finding_on_a_flagged_place_has_its_datum_replaced() {
    let (five, four) = common::builtins();
    for dates in [DateMode::Year, DateMode::Shift] {
        let mask = Mask::new(KEY, dates).unwrap();
        for (name, date) in [
            ("masked_findings_5010_sample.txt", "20241341"),
            ("masked_findings_4010_sample.txt", "241341"),
        ] {
            let bytes = common::load_fixture(name);
            let delims = oxedi_core::Delimiters::from_isa(&bytes).unwrap();
            let spec = common::select(&five, &four, &bytes, delims);
            let document = Document::with_delimiters(&bytes[..], delims).unwrap();
            let (_, plain) = Processor::run(spec, &document);
            let (_, masked) = Processor::run_masked(spec, &document, &mask);
            let datums = |found: &[Diagnostic]| -> Vec<String> {
                found
                    .iter()
                    .map(|d| String::from_utf8_lossy(&d.datum).into_owned())
                    .collect()
            };
            let plain = datums(&plain);
            let masked = datums(&masked);
            assert_eq!(plain.len(), 5, "{name}: {plain:?}");
            assert_eq!(masked.len(), plain.len(), "{name}");
            assert!(plain.iter().any(|d| d == date), "{name}");
            for (was, now) in plain.iter().zip(&masked) {
                assert_ne!(was, now, "{name} {dates:?}");
                assert!(is_token(now), "{name} {dates:?}: {now:?}");
            }
        }
    }
}
