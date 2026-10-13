//! The lists of fixtures and samples that the suites iterate hold every file
//! of their directories, so a new file cannot be skipped by them unnoticed.

mod common;

use std::collections::BTreeSet;
use std::path::Path;

/// The names of the files in `dir`, but the README.
fn files_in(dir: &Path) -> BTreeSet<String> {
    std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name != "README.md")
        .collect()
}

fn listed(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

#[test]
fn every_fixture_is_in_the_list() {
    let mut names = common::FIXTURES.to_vec();
    names.push(common::NO_ISA_FIXTURE);
    let expected = listed(&names);
    let on_disk = files_in(&common::fixtures_dir());
    let missing: Vec<_> = on_disk.difference(&expected).collect();
    let stale: Vec<_> = expected.difference(&on_disk).collect();
    assert!(
        missing.is_empty() && stale.is_empty(),
        "tests/fixtures files missing from common::FIXTURES: {missing:?}; \
         listed but not on disk: {stale:?}"
    );
}

#[test]
fn every_sample_is_in_the_list() {
    let expected = listed(common::SAMPLES);
    let on_disk = files_in(&common::samples_dir());
    let missing: Vec<_> = on_disk.difference(&expected).collect();
    let stale: Vec<_> = expected.difference(&on_disk).collect();
    assert!(
        missing.is_empty() && stale.is_empty(),
        "tests/samples files missing from common::SAMPLES: {missing:?}; \
         listed but not on disk: {stale:?}"
    );
}
