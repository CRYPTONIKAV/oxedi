"""Browser tests of the viewer page, in headless Chromium."""

from __future__ import annotations

import re

import oxedi
import pytest
from conftest import CSP, FIXTURES, SAMPLES

SAMPLE_FIXTURE = FIXTURES / "balanced_5010_sample.txt"
TWENTY_MB = 20 * 1024 * 1024


def text_of(viewer, selector: str) -> str:
    return viewer.page.inner_text(selector)


def test_sample_summary_tables_and_csv(viewer, tmp_path):
    viewer.open()
    viewer.load_sample()

    summary = text_of(viewer, "#panel-summary")
    assert "ACME HEALTH PLAN" in summary
    assert "SUNRISE CLINIC" in summary

    viewer.tab("tables")
    assert viewer.page.input_value("#table-select") == "claims"
    assert "4 rows" in text_of(viewer, "#panel-tables .toolbar")

    with viewer.page.expect_download() as download:
        viewer.page.click("text=Download CSV")
    path = tmp_path / "claims.csv"
    download.value.save_as(path)
    assert download.value.suggested_filename.endswith("-claims.csv")
    lines = path.read_bytes().decode("utf-8").split("\r\n")
    assert lines[-1] == ""
    assert len(lines) - 1 == 5, "a header and 4 data lines"
    assert "claim_id" in lines[0].split(",")
    assert all(line for line in lines[1:5])


def test_csv_neutralizes_formulas_and_keeps_negative_amounts(viewer, tmp_path):
    data = SAMPLE_FIXTURE.read_bytes()
    assert b"NM1*QC*1*DOE*" in data and b"CLP*CLM1001*1*300*" in data
    data = data.replace(b"NM1*QC*1*DOE*", b"NM1*QC*1*=DOE*", 1)
    data = data.replace(b"CLP*CLM1001*1*300*", b"CLP*CLM1001*1*-300*", 1)
    viewer.open()
    viewer.load("formulas.835", data)
    viewer.wait_loaded()
    viewer.tab("tables")

    with viewer.page.expect_download() as download:
        viewer.page.click("text=Download CSV")
    path = tmp_path / "claims.csv"
    download.value.save_as(path)
    text = path.read_bytes().decode("utf-8")
    first = next(line for line in text.split("\r\n") if "CLM1001" in line)
    cells = first.split(",")
    assert "'=DOE" in cells, first
    assert not any(c == "=DOE" for c in cells), "the formula cell must not be exported as is"
    assert any(re.fullmatch(r"-300(\.0+)?", c) for c in cells), first


def test_dropped_file_with_diagnostics_links_to_text(viewer):
    path = SAMPLES / "edi835_test_file.RMT"
    expected = oxedi.parse(path.read_bytes()).diagnostics
    assert expected, "the sample must have findings for this test"
    first = expected[0]

    viewer.open()
    viewer.drop(path.name, path.read_bytes())
    viewer.wait_loaded()
    assert text_of(viewer, "#diag-count") == str(len(expected))

    viewer.tab("diagnostics")
    assert text_of(viewer, "#panel-diagnostics .diag-list li:first-child .text") == str(first)
    viewer.page.click("#panel-diagnostics .diag-list li:first-child .jump")

    assert viewer.page.is_visible("#panel-text")
    current = viewer.page.locator("#panel-text .line.current")
    assert current.count() == 1
    assert current.locator(".no").inner_text() == str(first.segment)
    if first.element is not None:
        mark = current.locator("mark")
        assert mark.count() == 1
        assert mark.inner_text() != ""


def test_non_835_shows_the_core_error_then_the_sample_loads(viewer):
    with pytest.raises(Exception) as raised:
        oxedi.parse(b"%PDF-1.7")
    viewer.open()
    viewer.drop("report.pdf", b"%PDF-1.7")
    viewer.page.wait_for_selector("#alert:not([hidden])")
    alert = text_of(viewer, "#alert")
    assert alert == f"report.pdf: {raised.value}"
    assert "\n" not in alert.strip()
    assert viewer.page.is_hidden("#results")

    viewer.load_sample()
    assert viewer.page.is_hidden("#alert")
    assert "ACME HEALTH PLAN" in text_of(viewer, "#panel-summary")


def test_non_utf8_bytes_keep_offsets_aligned(viewer, tmp_path):
    data = SAMPLE_FIXTURE.read_bytes()
    assert b"SUNRISE" in data
    latin1 = tmp_path / "latin1.835"
    latin1.write_bytes(data.replace(b"SUNRISE", b"SUN\xd1ISE"))
    viewer.open()
    viewer.load(latin1)
    viewer.wait_loaded()

    assert "SUN�ISE CLINIC" in text_of(viewer, "#panel-summary")

    viewer.tab("tables")
    viewer.page.click("#panel-tables button[data-segment]")
    current = viewer.page.locator("#panel-text .line.current .raw")
    # Line breaks before a segment are shown as glyphs; the segment itself starts at its own id.
    shown = current.inner_text().lstrip("\u240d\u240a")
    assert shown.startswith("CLP*CLM1001"), "the segment after the Latin-1 byte is not shifted"


def test_csp_blocks_network_and_is_declared(viewer):
    viewer.expect_violations = True
    viewer.open()
    content = viewer.page.evaluate(
        "document.querySelector('meta[http-equiv=\"Content-Security-Policy\"]').content"
    )
    assert content == CSP
    rejected = viewer.page.evaluate(
        "fetch('/').then(() => false, () => true)"
    )
    assert rejected is True
    assert any("connect-src" in v for v in viewer.violations())


def test_largest_sample_is_shown_in_under_a_second(viewer):
    path = SAMPLES / "edi835_test_united.rmt"
    viewer.open()
    # Time from the input's change event to the results becoming visible.
    viewer.page.evaluate(
        """() => {
          window.__timing = new Promise((resolve) => {
            let t0 = 0;
            window.addEventListener('change', () => { t0 = performance.now(); }, true);
            new MutationObserver((_, obs) => {
              if (!document.getElementById('results').hidden) {
                obs.disconnect();
                resolve(performance.now() - t0);
              }
            }).observe(document.getElementById('results'), { attributes: true, attributeFilter: ['hidden'] });
          });
        }"""
    )
    viewer.load(path)
    elapsed = viewer.page.evaluate("window.__timing")
    print(f"\nviewer timing: {path.name} ({path.stat().st_size} bytes) shown in {elapsed:.0f} ms")
    assert elapsed < 1000, f"{elapsed:.0f} ms"


def test_size_guard_function(viewer):
    viewer.open()
    result = viewer.page.evaluate(
        """async (limit) => {
          const { needsConfirmation } = await import('/js/load.js');
          const big = new File([new Uint8Array(limit + 1)], 'big.835');
          const exact = new File([new Uint8Array(limit)], 'exact.835');
          const small = new File([new Uint8Array(100)], 'small.835');
          return [needsConfirmation(big), needsConfirmation(exact), needsConfirmation(small)];
        }""",
        TWENTY_MB,
    )
    assert result == [True, False, False]


def test_large_file_asks_before_parsing(viewer):
    viewer.open()
    viewer.load_sample()
    before = text_of(viewer, "#panel-summary")
    dialogs: list[tuple[str, str]] = []

    def dismiss(dialog) -> None:
        dialogs.append((dialog.type, dialog.message))
        dialog.dismiss()

    viewer.page.once("dialog", dismiss)
    viewer.load("big.835", bytes(TWENTY_MB + 1))
    viewer.page.wait_for_timeout(300)
    assert len(dialogs) == 1
    assert dialogs[0][0] == "confirm"
    assert "big.835" in dialogs[0][1] and "20.0 MB" in dialogs[0][1]
    assert viewer.page.is_visible("#results")
    assert text_of(viewer, "#panel-summary") == before
    assert viewer.page.is_hidden("#alert")
    assert viewer.page.is_hidden("#status")

    # Accepting parses it, and the (non-835) bytes give the core's error.
    viewer.page.once("dialog", lambda dialog: dialog.accept())
    viewer.load("big.835", bytes(TWENTY_MB + 1))
    viewer.page.wait_for_selector("#alert:not([hidden])")
    assert text_of(viewer, "#alert").startswith("big.835: ")


def test_unterminated_last_segment_is_visible_in_text(viewer):
    data = SAMPLE_FIXTURE.read_bytes().rstrip(b"\r\n")
    assert data.endswith(b"IEA*1*000000101~")
    data = data + b"ZZ*TAIL"
    viewer.open()
    viewer.load("unterminated.835", data)
    viewer.wait_loaded()
    viewer.tab("text")
    # The last row of the list holds the unterminated bytes.
    viewer.page.evaluate(
        "document.querySelector('#panel-text .textview').scrollTop = 1e9"
    )
    viewer.page.wait_for_selector("#panel-text .line .raw:text-is('ZZ*TAIL')")
