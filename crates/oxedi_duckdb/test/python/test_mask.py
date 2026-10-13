"""read_835 with mask_key against oxedi.parse_file with oxedi.Mask.

For every parsable input, every table and `diagnostics` must hold the same
rows as the masked parse, for each date mode.
"""

from __future__ import annotations

import pathlib
import re

import duckdb
import oxedi
import pytest

from test_oracle import (
    DIAGNOSTICS,
    PARSABLE,
    assert_same,
    con,  # noqa: F401  (fixture)
    file_id,
    oracle_diagnostics,
    oracle_table,
    read_835,
    tables_of,
)

KEY = "oxedi-test-key-0"
MODES = ["year", "shift", "keep"]


def masked_read(con: duckdb.DuckDBPyConnection, path: pathlib.Path, table: str, mode: str):
    relation = con.execute(
        "SELECT * FROM read_835(?, table_name := ?, binary := true, mask_key := ?, mask_dates := ?)",
        [str(path), table, KEY, mode],
    )
    return [column[0] for column in relation.description], [
        tuple(row) for row in relation.fetchall()
    ]


@pytest.mark.parametrize("mode", MODES)
@pytest.mark.parametrize("path", PARSABLE, ids=file_id)
def test_every_masked_table_equals_the_masked_parse(con, path, mode) -> None:  # noqa: F811
    result = oxedi.parse_file(str(path), mask=oxedi.Mask(KEY, dates=mode))
    for table in tables_of(result):
        if table == DIAGNOSTICS:
            expected = oracle_diagnostics(result)
        else:
            expected = oracle_table(result, table)
        assert_same(f"{file_id(path)} {table} {mode}", masked_read(con, path, table, mode), expected)


def test_a_masked_diagnostic_does_not_quote_the_patient(con, tmp_path) -> None:  # noqa: F811
    # An over-long member id in the patient segment is a finding whose datum
    # would quote it.
    source = pathlib.Path(__file__).resolve().parents[3] / "oxedi_core/tests/fixtures/balanced_5010_sample.txt"
    data = source.read_bytes()
    original = re.search(rb"NM1\*QC\*1\*[^*]*\*[^*]*\*[^*]*\*[^*]*\*[^*]*\*MI\*([^*~]+)", data)
    assert original, "the fixture has a patient NM1"
    member = original.group(1)
    long = member + b"X" * 100
    path = tmp_path / "long.835"
    path.write_bytes(data.replace(member, long, 1))
    plain = con.execute(
        "SELECT datum FROM read_835(?, table_name := 'diagnostics')", [str(path)]
    ).fetchall()
    assert any(long.decode() in (row[0] or "") for row in plain), "the finding quotes the id"
    rows = con.execute(
        "SELECT datum FROM read_835(?, table_name := 'diagnostics', mask_key := ?)",
        [str(path), KEY],
    ).fetchall()
    assert rows
    assert not any(member.decode() in (row[0] or "") for row in rows)
