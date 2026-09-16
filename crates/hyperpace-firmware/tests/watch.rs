//! Proves the watch route's constants and evaluation logic against real, archived vendor bytes,
//! never the live vendor site: `crates/hyperpace-firmware/src/watch.rs`'s module documentation
//! ("Network access from tests is forbidden") is enforced by only ever reading fixtures already
//! committed under `firmware/archive/other-devices/_web-driver-snapshots/`, read-only, the same
//! convention `tests/archive.rs` uses for the container parser.
//!
//! This is also the load-bearing proof that [`hyperpace_firmware::watch::CONFIG_TARGETS`]'s and
//! [`hyperpace_firmware::watch::RECORDED_SPA_INDEX_SHA256`]'s hardcoded sha256 values actually
//! match the archived files they were recomputed from, not just a number copied out of prose.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use hyperpace_firmware::watch::{
    CONFIG_TARGETS, DIRECTORY_TARGETS, DirectoryState, RECORDED_SPA_INDEX_SHA256, evaluate_config,
    evaluate_directory,
};

fn read_snapshot(relative: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../firmware/archive/other-devices/_web-driver-snapshots/web-snapshot-2026-09-15")
        .join(relative);
    std::fs::read(&path).unwrap_or_else(|err| {
        panic!("reading archived snapshot {}: {err}", path.display());
    })
}

#[test]
fn the_recorded_root_config_hash_matches_the_archived_snapshot() {
    let bytes = read_snapshot("www.lofree.tech/cfg.json");
    let target = CONFIG_TARGETS
        .iter()
        .find(|t| t.url == "https://www.lofree.tech/cfg.json")
        .expect("the root config target is registered");
    let finding = evaluate_config(target, &bytes);
    assert!(
        !finding.changed,
        "recorded sha256 must match the archived snapshot exactly: {finding}"
    );
    assert!(finding.evidence.is_empty());
}

#[test]
fn the_recorded_home_config_hash_matches_the_archived_snapshot() {
    let bytes = read_snapshot("www.lofree.tech/home/cfg.json");
    let target = CONFIG_TARGETS
        .iter()
        .find(|t| t.url == "https://www.lofree.tech/home/cfg.json")
        .expect("the home config target is registered");
    let finding = evaluate_config(target, &bytes);
    assert!(
        !finding.changed,
        "recorded sha256 must match the archived snapshot exactly: {finding}"
    );
}

#[test]
fn a_mutated_home_config_is_flagged_changed() {
    // Same file, one byte flipped: proves the comparison actually looks at content, not just
    // trusting the baseline blindly.
    let mut bytes = read_snapshot("www.lofree.tech/home/cfg.json");
    bytes.push(b'\n');
    let target = CONFIG_TARGETS
        .iter()
        .find(|t| t.url == "https://www.lofree.tech/home/cfg.json")
        .expect("the home config target is registered");
    let finding = evaluate_config(target, &bytes);
    assert!(finding.changed);
    // The archived home/cfg.json already carries a live "upgrade" block (dead Pulsar sample
    // links), so a change to it must surface that evidence again.
    assert!(!finding.evidence.is_empty());
    assert!(finding.evidence.iter().any(|s| s.contains("upgrade")));
}

#[test]
fn the_recorded_spa_index_hash_matches_the_archived_snapshot() {
    let bytes = read_snapshot("www.lofree.tech/index.html");
    assert_eq!(bytes.len(), 608, "the verdict records this page as 608 B");
    let target = DIRECTORY_TARGETS
        .first()
        .expect("at least one directory target is registered");
    let finding = evaluate_directory(target, 200, &bytes);
    assert_eq!(
        finding.state,
        DirectoryState::MissingAsRecorded,
        "the archived index must hash to RECORDED_SPA_INDEX_SHA256: {finding}"
    );
}

#[test]
fn a_different_real_page_on_the_same_host_is_not_mistaken_for_the_spa_fallback() {
    // www.lofree.tech/home/index.html is a real, different page on the same host (647 bytes,
    // different bundle references); a directory probe returning this instead of the true SPA
    // fallback must be flagged, not waved through because it is still "some www.lofree.tech page".
    let bytes = read_snapshot("www.lofree.tech/home/index.html");
    assert_ne!(hex_sha256(&bytes), RECORDED_SPA_INDEX_SHA256);
    let target = DIRECTORY_TARGETS
        .first()
        .expect("at least one directory target is registered");
    let finding = evaluate_directory(target, 200, &bytes);
    assert!(matches!(
        finding.state,
        DirectoryState::ContentChanged { .. }
    ));
}

/// Local hex-sha256, kept separate from the crate's own private helper of the same shape: this is
/// a test-only sanity check on the fixture, not part of the logic under test.
fn hex_sha256(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    use sha2::Digest as _;
    sha2::Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}
