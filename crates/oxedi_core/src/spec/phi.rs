//! Compilation of the columns flagged as patient data and of the seed column.

use super::Spec;
use super::error::SpecError;
use super::table_error::TableDefError;
use super::tables::TableDef;
use crate::column::ColumnType;
use crate::mask::Phi;
use crate::project::column_type;

/// Fills `table.phi` from the `phi` word each column wrote (parallel to
/// `table.columns`) and `table.phi_seed` from the `phi_seed` column name.
pub(super) fn compile_phi(
    spec: &Spec,
    table: &mut TableDef,
    written: &[Option<String>],
    seed: Option<&str>,
) -> Result<(), SpecError> {
    let fail = |column: Option<&str>, reason| SpecError::BadTable {
        table: table.name.clone(),
        column: column.map(str::to_string),
        reason,
    };
    let mut flags = Vec::with_capacity(table.columns.len());
    for ((name, source), word) in table.columns.iter().zip(written) {
        let Some(word) = word else {
            flags.push(None);
            continue;
        };
        let Some(kind) = Phi::from_word(word) else {
            return Err(fail(
                Some(name),
                TableDefError::UnknownPhi {
                    given: word.clone(),
                },
            ));
        };
        let found = column_type(spec, table, source);
        let reason = match (kind, found) {
            (Phi::Date, ColumnType::Date32) | (Phi::Name | Phi::Id, ColumnType::Binary) => None,
            (Phi::Date, found) => Some(TableDefError::PhiNeedsDate { found }),
            (kind, found) => Some(TableDefError::PhiNeedsText {
                kind: kind.word(),
                found,
            }),
        };
        if let Some(reason) = reason {
            return Err(fail(Some(name), reason));
        }
        flags.push(Some(kind));
    }
    let seed_index = match seed {
        None => None,
        Some(name) => {
            let Some(index) = table.columns.iter().position(|(column, _)| column == name) else {
                return Err(fail(
                    None,
                    TableDefError::UnknownPhiSeed {
                        name: name.to_string(),
                    },
                ));
            };
            if flags.get(index).copied().flatten() != Some(Phi::Id) {
                return Err(fail(
                    Some(name),
                    TableDefError::PhiSeedNotId {
                        name: name.to_string(),
                    },
                ));
            }
            Some(index)
        }
    };
    table.phi = flags;
    table.phi_seed = seed_index;
    Ok(())
}
