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

/// The quoted names in the bench workload list.
fn bench_samples() -> Vec<String> {
    let source = include_str!("../benches/common/mod.rs");
    let start = source
        .find("pub const SAMPLES: &[&str] = &[")
        .expect("benches/common/mod.rs lists SAMPLES");
    let body = &source[start..];
    let body = &body[..body.find("];").expect("the SAMPLES list ends")];
    body.split('"')
        .skip(1)
        .step_by(2)
        .map(String::from)
        .collect()
}

/// The benches run the engine on the largest samples, so their list must be the
/// largest samples by size, not an arbitrary subset.
#[test]
fn the_bench_list_is_the_largest_samples() {
    let dir = common::samples_dir();
    let mut sizes: Vec<(u64, String)> = files_in(&dir)
        .into_iter()
        .map(|name| {
            let len = std::fs::metadata(dir.join(&name))
                .unwrap_or_else(|e| panic!("cannot stat {name}: {e}"))
                .len();
            (len, name)
        })
        .collect();
    sizes.sort_by(|a, b| b.cmp(a));
    let mut largest: Vec<String> = sizes.into_iter().take(3).map(|(_, name)| name).collect();
    largest.sort();
    let mut listed = bench_samples();
    listed.sort();
    assert_eq!(
        listed, largest,
        "benches/common SAMPLES must be the three largest files of tests/samples"
    );
}
