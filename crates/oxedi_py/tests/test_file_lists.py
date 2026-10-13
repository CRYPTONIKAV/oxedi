"""The lists of fixtures and samples hold every file of their directories."""

from conftest import CORE_TESTS, FIXTURES, SAMPLES


def test_every_fixture_and_sample_is_in_the_lists():
    """A file in the fixtures or samples directory that the lists leave out
    would be skipped by every suite that iterates them."""
    for folder, listed in (("fixtures", FIXTURES), ("samples", SAMPLES)):
        on_disk = {p.name for p in (CORE_TESTS / folder).iterdir() if p.name != "README.md"}
        missing = sorted(on_disk - set(listed))
        stale = sorted(set(listed) - on_disk)
        assert not missing and not stale, (
            f"tests/{folder}: files missing from the list in conftest.py: {missing}; "
            f"listed but not on disk: {stale}"
        )
