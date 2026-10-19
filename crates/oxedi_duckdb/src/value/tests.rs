use libduckdb_sys as ffi;

use super::{primitive_name, primitive_name_or_id};

#[test]
fn primitive_names_are_sql_names() {
    assert_eq!(
        primitive_name(ffi::DUCKDB_TYPE_DUCKDB_TYPE_INTEGER),
        "INTEGER"
    );
    assert_eq!(
        primitive_name(ffi::DUCKDB_TYPE_DUCKDB_TYPE_VARCHAR),
        "VARCHAR"
    );
    assert_eq!(
        primitive_name(ffi::DUCKDB_TYPE_DUCKDB_TYPE_BOOLEAN),
        "BOOLEAN"
    );
    assert_eq!(primitive_name(ffi::DUCKDB_TYPE_DUCKDB_TYPE_BIT), "BIT");
    assert_eq!(
        primitive_name(ffi::DUCKDB_TYPE_DUCKDB_TYPE_BIGNUM),
        "BIGNUM"
    );
    assert_eq!(
        primitive_name(ffi::DUCKDB_TYPE_DUCKDB_TYPE_GEOMETRY),
        "GEOMETRY"
    );
    assert_eq!(
        primitive_name(ffi::DUCKDB_TYPE_DUCKDB_TYPE_VARIANT),
        "VARIANT"
    );
    assert_eq!(primitive_name(9999), "a type without a primitive name");
}

#[test]
fn an_unnamed_type_is_refused_with_its_id() {
    // DuckDB adds kinds across versions, and those have no name here yet: the
    // refusal says which type id to name instead of dead-ending on a sentence.
    assert_eq!(
        primitive_name_or_id(9999),
        "a type without a primitive name (DuckDB type id 9999)"
    );
    // Named kinds are unaffected: they still answer with their SQL name.
    assert_eq!(
        primitive_name_or_id(ffi::DUCKDB_TYPE_DUCKDB_TYPE_INTEGER),
        "INTEGER"
    );
    assert_eq!(
        primitive_name_or_id(ffi::DUCKDB_TYPE_DUCKDB_TYPE_BIT),
        "BIT"
    );
}
