//! The arguments of `read_835`, read from the bind info.

use std::ffi::CString;

use libduckdb_sys as ffi;
use oxedi_core::{DateMode, Mask};

use crate::error::ReadError;
use crate::value::Value;

/// The table read when `table_name` is not given.
pub const DEFAULT_TABLE: &str = "claims";

/// The named parameters, in registration order, with whether each is a
/// boolean (else a VARCHAR).
pub const NAMED: [(&str, bool); 7] = [
    ("table_name", false),
    ("filename", true),
    ("version", false),
    ("binary", true),
    ("ignore_errors", true),
    ("mask_key", false),
    ("mask_dates", false),
];

/// What one call of `read_835` asked for.
#[derive(Debug, Clone)]
pub struct Options {
    /// The paths as given: plain paths or glob patterns.
    pub paths: Vec<String>,
    /// The table to return.
    pub table_name: String,
    /// Whether to add a `filename` column.
    pub filename: bool,
    /// The forced spec version, if any.
    pub version: Option<String>,
    /// Whether text columns are `BLOB` instead of `VARCHAR`.
    pub binary: bool,
    /// Whether a file that cannot be parsed is reported instead of failing.
    pub ignore_errors: bool,
    /// The mask that hides patient data, when `mask_key` was given.
    pub mask: Option<Mask>,
}

/// A named parameter as the call gave it.
pub(crate) enum Named {
    /// Not given.
    Absent,
    /// Given as SQL NULL.
    Null,
    /// Given with a value.
    Given(Value),
}

impl Options {
    /// Reads the arguments of the bind in progress.
    ///
    /// # Safety
    ///
    /// `info` must be the bind info DuckDB passed to the running bind
    /// callback of a function registered with one positional parameter and
    /// the named parameters of [`NAMED`].
    pub unsafe fn of_bind(info: ffi::duckdb_bind_info) -> Result<Options, ReadError> {
        // SAFETY: index 0 is the one positional parameter (caller
        // contract); the value is owned by the caller and taken by `owned`.
        let path = unsafe { Value::owned(ffi::duckdb_bind_get_parameter(info, 0)) };
        let paths = match path {
            Some(path) => paths_of(&path)?,
            None => return Err(ReadError::NullPath { position: None }),
        };
        // SAFETY: each name below is one of NAMED (caller contract).
        let named = |name: &str| unsafe { named(info, name) };
        let text = |name: &'static str, parameter: Named| match parameter {
            Named::Absent => Ok(None),
            Named::Null => Err(ReadError::NullOption { name }),
            Named::Given(value) => constant(name, value.text()).map(Some),
        };
        let flag = |name: &'static str| match named(name) {
            Named::Absent => Ok(false),
            Named::Null => Err(ReadError::NullOption { name }),
            Named::Given(value) => Ok(value.boolean()),
        };
        Ok(Options {
            paths,
            table_name: match named("table_name") {
                Named::Absent => DEFAULT_TABLE.to_owned(),
                Named::Null => return Err(ReadError::NullOption { name: "table_name" }),
                Named::Given(value) => constant("table_name", value.text())?,
            },
            filename: flag("filename")?,
            version: match named("version") {
                Named::Absent | Named::Null => None,
                Named::Given(value) => Some(constant("version", value.text())?),
            },
            binary: flag("binary")?,
            ignore_errors: flag("ignore_errors")?,
            mask: mask_of(
                text("mask_key", named("mask_key"))?,
                text("mask_dates", named("mask_dates"))?,
            )?,
        })
    }
}

/// The mask the key and date mode ask for: none without a key, and a date
/// mode without a key is refused.
fn mask_of(key: Option<String>, dates: Option<String>) -> Result<Option<Mask>, ReadError> {
    let Some(key) = key else {
        return match dates {
            Some(dates) => Err(ReadError::MaskDatesWithoutKey { dates }),
            None => Ok(None),
        };
    };
    let dates = match dates {
        Some(dates) => DateMode::parse(&dates).map_err(|source| ReadError::Mask {
            name: "mask_dates",
            source,
        })?,
        None => DateMode::Year,
    };
    Mask::new(key.as_bytes(), dates)
        .map(Some)
        .map_err(|source| ReadError::Mask {
            name: "mask_key",
            source,
        })
}

/// The named parameter `name` of the bind.
///
/// # Safety
///
/// `info` must be a live bind info.
pub(crate) unsafe fn named(info: ffi::duckdb_bind_info, name: &str) -> Named {
    let Ok(name) = CString::new(name) else {
        return Named::Absent;
    };
    // SAFETY: `info` is live (caller contract) and `name` is a C string; the
    // value returned (null when the parameter was not given) is owned here.
    let raw = unsafe { ffi::duckdb_bind_get_named_parameter(info, name.as_ptr()) };
    if raw.is_null() {
        return Named::Absent;
    }
    // SAFETY: `raw` is a live value handle owned here.
    match unsafe { Value::owned(raw) } {
        Some(value) => Named::Given(value),
        None => Named::Null,
    }
}

/// The prefix of the name DuckDB gives a table macro's parameters while it
/// binds the macro's body.
const MACRO_PARAMETERS: &str = "0_macro_parameters";

/// The text of the named parameter `name`, refused when it is a table
/// macro's parameter rather than a value.
///
/// When a table macro is created, DuckDB binds its body once. A table
/// function argument that refers to a macro parameter normally defers that
/// bind to the macro's call, but a bare parameter given to a named parameter
/// with `:=` or `=>` is not recognized: DuckDB passes its qualified name,
/// `0_macro_parameters<macro>.<parameter>`, as a string constant. The C API
/// hands the bind only that string, and a bind error cannot ask DuckDB to
/// defer, so the prefix is the only sign that the value is not a constant.
/// A real table name or version never starts with it.
fn constant(name: &'static str, text: String) -> Result<String, ReadError> {
    match macro_parameter(&text) {
        Some(parameter) => Err(ReadError::MacroParameter {
            name,
            parameter,
            value: text,
        }),
        None => Ok(text),
    }
}

/// The name of the table macro parameter `text` stands for, when it is one
/// (see [`constant`]).
pub(crate) fn macro_parameter(text: &str) -> Option<String> {
    text.strip_prefix(MACRO_PARAMETERS)
        .and_then(|qualified| qualified.rsplit_once('.'))
        .map(|(_, parameter)| parameter.to_owned())
}

/// The paths of a VARCHAR or a list of VARCHAR.
fn paths_of(value: &Value) -> Result<Vec<String>, ReadError> {
    match value.type_id() {
        ffi::DUCKDB_TYPE_DUCKDB_TYPE_VARCHAR => Ok(vec![value.text()]),
        ffi::DUCKDB_TYPE_DUCKDB_TYPE_LIST => {
            let items = value.list();
            if items.is_empty() {
                return Err(ReadError::EmptyPathList);
            }
            items
                .iter()
                .enumerate()
                .map(|(index, item)| match item {
                    None => Err(ReadError::NullPath {
                        position: Some(index + 1),
                    }),
                    Some(item) if item.type_id() == ffi::DUCKDB_TYPE_DUCKDB_TYPE_VARCHAR => {
                        Ok(item.text())
                    }
                    Some(_) => Err(ReadError::PathType {
                        found: value.type_name(),
                    }),
                })
                .collect()
        }
        _ => Err(ReadError::PathType {
            found: value.type_name(),
        }),
    }
}

#[cfg(test)]
mod tests;
