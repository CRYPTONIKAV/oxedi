import sys

import pyarrow as pa
import pytest

from conftest import ALL_FILES, parse_named
from oxedi import Spec

PARENT = b"oxedi.parent"


def without_parent(metadata):
    rest = {key: value for key, value in (metadata or {}).items() if key != PARENT}
    return rest or None


@pytest.mark.parametrize("name", ALL_FILES)
def test_the_schema_is_the_schema_of_the_parsed_tables(name):
    tables = parse_named(name).tables
    schemas = tables.spec.schema()
    assert list(schemas) == tables.keys()
    for table_name in tables.keys():
        parsed = pa.table(tables[table_name]).schema
        expected = schemas[table_name]
        assert expected.equals(parsed, check_metadata=False), table_name
        for field, parsed_field in zip(expected, parsed):
            assert field.nullable == parsed_field.nullable
            assert without_parent(field.metadata) == parsed_field.metadata, (
                table_name,
                field.name,
            )


@pytest.mark.parametrize("version", ["5010", "4010"])
def test_reference_columns_name_the_table_they_point_at(version):
    schemas = Spec.builtin(version).schema()
    parents = {
        name: {
            field.name: field.metadata[PARENT].decode()
            for field in schema
            if field.metadata and PARENT in field.metadata
        }
        for name, schema in schemas.items()
    }
    assert parents["payments"] == {}
    assert parents["claims"] == {"payment": "payments"}
    assert parents["services"] == {"payment": "payments", "claim": "claims"}
    assert parents["adjustments"] == {
        "payment": "payments",
        "claim": "claims",
        "service": "services",
    }
    assert parents["provider_adjustments"] == {"payment": "payments"}
    claims = schemas["claims"]
    assert claims.names[:3] == ["row", "segment", "payment"]
    assert claims.field("claim_id").type == pa.binary()
    assert claims.field("charge_amount").type == pa.decimal128(38, 2)
    assert claims.field("statement_from").type == pa.date32()


def test_a_patched_spec_describes_its_own_tables():
    spec = Spec.builtin().patch({
        "tables": {
            "groups": {
                "loops": ["group"],
                "ref": "group",
                "columns": {
                    "control": {"segment": "GS", "element": 6},
                    "date": {"segment": "GS", "element": 4},
                    "time": {"segment": "GS", "element": 5},
                },
            }
        }
    })
    schemas = spec.schema()
    groups = schemas["groups"]
    assert groups.names == ["row", "segment", "control", "date", "time"]
    assert groups.field("control").type == pa.int64()
    assert groups.field("date").type == pa.date32()
    assert groups.field("time").type == pa.time32("s")
    assert schemas["payments"].field("group").metadata == {PARENT: b"groups"}
    assert "groups" not in Spec.builtin().schema()


def test_a_missing_pyarrow_is_named(monkeypatch):
    monkeypatch.setitem(sys.modules, "pyarrow", None)
    with pytest.raises(ImportError) as info:
        Spec.builtin().schema()
    assert str(info.value) == (
        "Spec.schema needs pyarrow, which is not installed; "
        'install it with: pip install "oxedi[pandas]"'
    )
    assert isinstance(info.value.__cause__, ImportError)
