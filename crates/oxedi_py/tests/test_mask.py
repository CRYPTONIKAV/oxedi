"""``oxedi.Mask``: patient data hidden in the tables and diagnostics."""

import importlib.util
import re

import pyarrow
import pytest

import oxedi
from conftest import (
    ALL_FILES,
    GOLDEN,
    SUMMARY_ONLY,
    delimiters_for,
    read,
)
from test_write import CLEAN, envelope, pyx12_findings

KEY = b"oxedi-test-key-0"
MASKED = GOLDEN / "masked"

requires_pyx12 = pytest.mark.skipif(
    importlib.util.find_spec("pyx12") is None, reason="pyx12 is not installed"
)


def rows(table):
    return pyarrow.table(table).to_pylist()


def as_text(value):
    return value.decode() if isinstance(value, bytes) else value


def masked(name, mask=None):
    return oxedi.parse(
        read(name), delimiters=delimiters_for(name), mask=mask or oxedi.Mask(KEY)
    )


def test_masked_tables_render_as_the_golden_files(file_name):
    tables = masked(file_name).tables
    if file_name in SUMMARY_ONLY:
        actual = "".join(f"{name} rows: {len(tables[name])}\n" for name in tables)
        golden = MASKED / f"{file_name}.tables.summary.txt"
    else:
        actual = tables.render()
        golden = MASKED / f"{file_name}.tables.txt"
    assert actual == golden.read_text(encoding="utf-8")


def test_masked_diagnostics_display_as_the_golden_files(file_name):
    diagnostics = masked(file_name).diagnostics
    golden = MASKED / f"{file_name}.diagnostics.txt"
    assert "".join(f"{d}\n" for d in diagnostics) == golden.read_text(
        encoding="utf-8"
    )


def test_the_shift_mode_renders_as_its_golden_files():
    name = "balanced_5010_sample.txt"
    result = masked(name, oxedi.Mask(KEY, dates="shift"))
    assert result.tables.render() == (MASKED / f"{name}.shift.tables.txt").read_text(
        encoding="utf-8"
    )
    assert "".join(f"{d}\n" for d in result.diagnostics) == (
        MASKED / f"{name}.shift.diagnostics.txt"
    ).read_text(encoding="utf-8")


def test_a_str_key_is_its_utf8_bytes():
    name = "emedny_sample.txt"
    assert (
        masked(name, oxedi.Mask(KEY.decode())).tables.render()
        == masked(name).tables.render()
    )


def test_stream_batches_concatenate_to_the_masked_parse(file_name):
    mask = oxedi.Mask(KEY)
    whole = masked(file_name, mask)
    batches = list(
        oxedi.stream(read(file_name), delimiters=delimiters_for(file_name), mask=mask)
    )
    for name in whole.tables.keys():
        got = [row for batch in batches for row in rows(batch.tables[name])]
        assert got == rows(whole.tables[name])
    assert [str(d) for b in batches for d in b.diagnostics] == [
        str(d) for d in whole.diagnostics
    ]


def test_parse_file_masks(tmp_path):
    name = "emedny_sample.txt"
    path = tmp_path / name
    path.write_bytes(read(name))
    mask = oxedi.Mask(KEY)
    assert (
        oxedi.parse_file(path, mask=mask).tables.render()
        == masked(name, mask).tables.render()
    )


def test_a_short_key_is_refused_with_the_core_message():
    with pytest.raises(ValueError) as info:
        oxedi.Mask(b"x" * 15)
    assert str(info.value) == "the mask key must be at least 16 bytes long; got 15"


def test_an_empty_key_is_refused_with_the_core_message():
    with pytest.raises(ValueError) as info:
        oxedi.Mask(b"")
    assert str(info.value) == "the mask key must be at least 16 bytes long; got 0"


def test_an_unknown_date_mode_is_refused():
    with pytest.raises(ValueError) as info:
        oxedi.Mask(KEY, dates="month")
    assert str(info.value) == (
        'the mask date mode must be "year", "shift" or "keep"; got "month"'
    )


def test_the_repr_does_not_show_the_key():
    shown = repr(oxedi.Mask(KEY))
    assert KEY.decode() not in shown and "oxedi-test" not in shown
    assert "year" in shown


def test_no_masked_patient_value_survives_in_cells_or_diagnostics():
    for name in ["emedny_sample.txt", "balanced_5010_sample.txt"]:
        plain = oxedi.parse(read(name), delimiters=delimiters_for(name))
        result = masked(name)
        ids = {
            as_text(row["patient_id"])
            for row in rows(plain.tables["claims"])
            if row["patient_id"] and len(row["patient_id"]) >= 6
        }
        assert ids
        rendered = result.tables.render() + "".join(
            f"{d}\n" for d in result.diagnostics
        )
        for value in ids:
            assert value not in rendered, (name, value)
        assert all(
            re.fullmatch("[0-9a-f]{16}", as_text(row["patient_id"]))
            for row in rows(result.tables["claims"])
            if row["patient_id"]
        )


@pytest.mark.parametrize("name", CLEAN)
def test_masked_tables_write_back_without_findings(name):
    result = masked(name)
    data = oxedi.write(result.tables, envelope())
    again = oxedi.parse(data)
    assert again.diagnostics == []


@requires_pyx12
@pytest.mark.parametrize("name", CLEAN)
def test_pyx12_finds_nothing_new_in_the_masked_written_file(name):
    original = pyx12_findings(oxedi.write(oxedi.parse(read(name), delimiters=delimiters_for(name)).tables, envelope()))
    written = pyx12_findings(oxedi.write(masked(name).tables, envelope()))
    assert not (written - original)
