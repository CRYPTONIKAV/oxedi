"""The block finder: every fence form it accepts, and the ones it must reject."""

import pytest

from blocks import parse


def blocks(text, lang="python"):
    page = parse(text, "page.md")
    return page, [b.code for b in page.run.get(lang, [])]


@pytest.mark.parametrize(
    "fence, close",
    [
        ("```python", "```"),
        ("```py", "```"),
        ("```python3", "```"),
        ('```python title="x.py"', "```"),
        ("``` python", "```"),
        ("```{.python}", "```"),
        ("~~~python", "~~~"),
        ("````python", "````"),
    ],
)
def test_python_fence_forms_run(fence, close):
    page, found = blocks(f"text\n\n{fence}\nx = 1\n{close}\n")
    assert found == ["x = 1\n"]
    assert page.fenced == 1 and not page.problems


def test_sql_fence_with_attributes_runs():
    _, found = blocks('```sql title="q.sql"\nSELECT 1;\n```\n', "sql")
    assert found == ["SELECT 1;\n"]


def test_indented_fences_are_dedented():
    text = '!!! note\n    ```python\n    if True:\n        x = 1\n    ```\n'
    page, found = blocks(text)
    assert found == ["if True:\n    x = 1\n"]
    assert page.fenced == 1


def test_other_languages_are_ignored_and_their_content_is_not_read():
    page, found = blocks("```bash\necho hi\n```\n\n```json\n{}\n```\n\n```python\ny = 2\n```\n")
    assert found == ["y = 2\n"]
    assert page.fenced == 1


def test_pycon_is_a_problem():
    page, _ = blocks("```pycon\n>>> 1\n```\n")
    assert page.problems and "pycon" in page.problems[0] and page.fenced == 1


def test_skip_marker_with_reason_skips_and_is_listed():
    page, found = blocks("<!-- docs-test: skip needs the network -->\n```python\nz = 3\n```\n")
    assert found == []
    assert page.skipped == [("python", "needs the network", "z = 3")]
    assert not page.problems


def test_skip_marker_works_on_tilde_and_indented_fences():
    page, found = blocks("  <!-- docs-test: skip why -->\n  ~~~py\n  z = 3\n  ~~~\n")
    assert found == [] and len(page.skipped) == 1


def test_skip_marker_without_reason_is_a_problem():
    page, _ = blocks("<!-- docs-test: skip -->\n```python\nz = 3\n```\n")
    assert any("needs a reason" in p for p in page.problems)


def test_skip_marker_not_before_a_fence_is_a_problem():
    page, _ = blocks("<!-- docs-test: skip why -->\n\n```python\nz = 3\n```\n")
    assert any("right before" in p for p in page.problems)


def test_block_records_its_line():
    page, _ = blocks("a\n\n```python\nx = 1\n```\n")
    assert page.run["python"][0].line == 3


def test_unclosed_fence_is_a_problem():
    page, _ = blocks("```python\nx = 1\n")
    assert any("never closed" in p for p in page.problems)
