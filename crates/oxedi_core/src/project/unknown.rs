use std::fmt;

use crate::spec::Spec;

/// A table name that the spec does not define, asked for when choosing which
/// tables to fill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTable {
    /// The name as the caller wrote it.
    pub name: String,
    /// The names of the tables the spec defines, in spec order.
    pub tables: Vec<String>,
}

impl UnknownTable {
    /// The error for `name` against the tables of `spec`.
    pub(crate) fn new(spec: &Spec, name: &str) -> Self {
        Self {
            name: name.to_owned(),
            tables: spec.tables().iter().map(|def| def.name.clone()).collect(),
        }
    }
}

impl fmt::Display for UnknownTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let quoted: Vec<String> = self.tables.iter().map(|name| format!("{name:?}")).collect();
        write!(
            f,
            "table {:?} is not defined by the spec, so it cannot be filled; its tables are {}",
            self.name,
            quoted.join(", ")
        )
    }
}

impl std::error::Error for UnknownTable {}
