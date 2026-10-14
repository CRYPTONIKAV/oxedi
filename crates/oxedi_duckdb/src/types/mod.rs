//! The DuckDB type of each core column type, and owned logical type
//! handles: those DuckDB is given and those it hands back.

use std::fmt;

use libduckdb_sys as ffi;
use oxedi_core::ColumnType;

use crate::error::ReadError;

/// Decimal widths stored as a 128-bit integer in a DuckDB vector.
const HUGEINT_WIDTHS: std::ops::RangeInclusive<u8> = 19..=38;

/// The DuckDB type of one output column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlType {
    /// `BIGINT`, from `int64` (its implied scale is not applied).
    Bigint,
    /// `VARCHAR`, from `binary` text that is valid UTF-8.
    Varchar,
    /// `BLOB`, from `binary` with `binary := true`.
    Blob,
    /// `DATE`, from `date32`.
    Date,
    /// `TIME`, from `time32` seconds.
    Time,
    /// `DECIMAL(width, scale)` stored as a 128-bit integer, from `decimal128`.
    Decimal {
        /// Total digits.
        width: u8,
        /// Decimal places.
        scale: u8,
    },
}

impl SqlType {
    /// The DuckDB type of a core column; `binary` turns text into `BLOB`.
    pub fn of(
        table: &str,
        column: &str,
        kind: ColumnType,
        binary: bool,
    ) -> Result<SqlType, ReadError> {
        SqlType::try_of(kind, binary).ok_or_else(|| ReadError::UnsupportedType {
            table: table.to_owned(),
            column: column.to_owned(),
            kind,
        })
    }

    /// The DuckDB type of a core column; `binary` turns text into `BLOB`.
    /// `None` for a decimal DuckDB does not store as a 128-bit integer.
    pub fn try_of(kind: ColumnType, binary: bool) -> Option<SqlType> {
        match kind {
            ColumnType::Binary if binary => Some(SqlType::Blob),
            ColumnType::Binary => Some(SqlType::Varchar),
            ColumnType::Int64 { .. } => Some(SqlType::Bigint),
            ColumnType::Decimal128 { precision, scale }
                if HUGEINT_WIDTHS.contains(&precision) && scale <= precision =>
            {
                Some(SqlType::Decimal {
                    width: precision,
                    scale,
                })
            }
            ColumnType::Decimal128 { .. } => None,
            ColumnType::Date32 => Some(SqlType::Date),
            ColumnType::Time32 => Some(SqlType::Time),
        }
    }

    /// A new logical type handle for this type.
    pub fn logical_type(self) -> LogicalType {
        let raw = match self {
            // SAFETY: creating a logical type from a primitive id has no
            // precondition; the handle is destroyed by `LogicalType`.
            SqlType::Bigint => unsafe {
                ffi::duckdb_create_logical_type(ffi::DUCKDB_TYPE_DUCKDB_TYPE_BIGINT)
            },
            // SAFETY: as above.
            SqlType::Varchar => unsafe {
                ffi::duckdb_create_logical_type(ffi::DUCKDB_TYPE_DUCKDB_TYPE_VARCHAR)
            },
            // SAFETY: as above.
            SqlType::Blob => unsafe {
                ffi::duckdb_create_logical_type(ffi::DUCKDB_TYPE_DUCKDB_TYPE_BLOB)
            },
            // SAFETY: as above.
            SqlType::Date => unsafe {
                ffi::duckdb_create_logical_type(ffi::DUCKDB_TYPE_DUCKDB_TYPE_DATE)
            },
            // SAFETY: as above.
            SqlType::Time => unsafe {
                ffi::duckdb_create_logical_type(ffi::DUCKDB_TYPE_DUCKDB_TYPE_TIME)
            },
            // SAFETY: width and scale were checked in `SqlType::of` (scale
            // at most width, width at most 38).
            SqlType::Decimal { width, scale } => unsafe {
                ffi::duckdb_create_decimal_type(width, scale)
            },
        };
        LogicalType(raw)
    }
}

impl fmt::Display for SqlType {
    /// The type as DuckDB's `DESCRIBE` writes it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlType::Bigint => write!(f, "BIGINT"),
            SqlType::Varchar => write!(f, "VARCHAR"),
            SqlType::Blob => write!(f, "BLOB"),
            SqlType::Date => write!(f, "DATE"),
            SqlType::Time => write!(f, "TIME"),
            SqlType::Decimal { width, scale } => write!(f, "DECIMAL({width},{scale})"),
        }
    }
}

/// The `ANY` type, for the path parameter.
pub fn any_type() -> LogicalType {
    // SAFETY: creating a logical type from a primitive id has no precondition.
    LogicalType(unsafe { ffi::duckdb_create_logical_type(ffi::DUCKDB_TYPE_DUCKDB_TYPE_ANY) })
}

/// The `VARCHAR` type.
pub fn varchar_type() -> LogicalType {
    SqlType::Varchar.logical_type()
}

/// The `INTEGER` type.
pub fn integer_type() -> LogicalType {
    // SAFETY: creating a logical type from a primitive id has no precondition.
    LogicalType(unsafe { ffi::duckdb_create_logical_type(ffi::DUCKDB_TYPE_DUCKDB_TYPE_INTEGER) })
}

/// The `BOOLEAN` type.
pub fn boolean_type() -> LogicalType {
    // SAFETY: creating a logical type from a primitive id has no precondition.
    LogicalType(unsafe { ffi::duckdb_create_logical_type(ffi::DUCKDB_TYPE_DUCKDB_TYPE_BOOLEAN) })
}

/// An owned DuckDB logical type handle, destroyed on drop.
#[derive(Debug)]
pub struct LogicalType(ffi::duckdb_logical_type);

impl LogicalType {
    /// Takes ownership of a logical type handle; `None` for a null handle.
    ///
    /// # Safety
    ///
    /// `raw` must be null or a logical type handle the caller owns and
    /// gives up.
    pub unsafe fn owned(raw: ffi::duckdb_logical_type) -> Option<LogicalType> {
        (!raw.is_null()).then_some(LogicalType(raw))
    }

    /// The raw handle, valid while `self` lives.
    pub fn raw(&self) -> ffi::duckdb_logical_type {
        self.0
    }
}

impl Drop for LogicalType {
    fn drop(&mut self) {
        // SAFETY: the handle was created by DuckDB and is destroyed once.
        unsafe { ffi::duckdb_destroy_logical_type(&mut self.0) };
    }
}

#[cfg(test)]
mod tests;
