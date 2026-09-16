//! Route 1 of the plan's three firmware acquisition routes ("watch for publication"): watch the
//! vendor's own properties for firmware appearing, and report what changed.
//! `docs/papers/hyperpace-firmware-platform/FIRMWARE-VERDICT.md` names the exact URLs, the
//! directory oracle's content-decides-not-status-code signal (section 3c, candidates 22 to 24, and
//! the bottom-line table's `www.lofree.tech/mouse/cfg.json` paragraph), and the recorded sha256
//! values this module checks fetched bytes against.
//!
//! Every function here is pure: it takes bytes a caller already fetched and returns a finding.
//! Nothing in this module performs I/O (the crate's own documentation: "never opens a device
//! itself" extends here to never fetching a URL itself); `hyperpace-app` owns the actual HTTP
//! fetch and calls [`evaluate_config`] and [`evaluate_directory`] with what it received, exactly
//! as it owns the local file I/O around [`crate::package::Package::parse`] for the import route.
//! This module also never downloads or acts on a `.bin` link it notices inside a config file: it
//! only reports that the link's text is now present, quoted, for a human to act on.
//!
//! # What a finding here can and cannot prove
//!
//! A finding is evidence a human should look at, never a firmware discovery. A changed sha256 is
//! necessary but not sufficient evidence that firmware exists ([`EVIDENCE_DISCLAIMER`]); nothing
//! here ever returns a "firmware available" verdict, only what literally changed, quoted from the
//! fetched bytes.

use core::fmt;

use sha2::{Digest, Sha256};

/// What every finding in this module says, verbatim or paraphrased, whenever it reports a change:
/// a byte change alone does not prove firmware exists. Every caller surfacing a finding (a log
/// line, a DTO, a UI string) must carry this, never render a change as "firmware found".
pub const EVIDENCE_DISCLAIMER: &str =
    "a changed file is evidence worth checking by hand, not proof that firmware is available";

/// sha256 of `www.lofree.tech`'s single-page-app fallback index, served for any path nginx does
/// not otherwise resolve on that host. `FIRMWARE-VERDICT.md` candidate 32 records it abbreviated
/// ("608 B, sha256 9421338c..."); this is the full digest, recomputed from the archived snapshot
/// `firmware/archive/other-devices/_web-driver-snapshots/web-snapshot-2026-09-15/www.lofree.tech/
/// index.html` (608 bytes, matching the verdict's own recorded size).
pub const RECORDED_SPA_INDEX_SHA256: &str =
    "9421338c17dec8929880e09a3619b7d0b76cea64c87c98e7b5ac7d0e6df25bb1";

/// One vendor configuration file this module watches, with the sha256 the 2026-09-16 research
/// recorded for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigTarget {
    /// Full URL to fetch.
    pub url: &'static str,
    /// What this file is, for display.
    pub description: &'static str,
    /// sha256 recorded for this file's content as of the research (hex, lowercase).
    pub recorded_sha256: &'static str,
}

/// The two configuration files the verdict names: the production Lofree HYPACE driver's own
/// config (`FIRMWARE-VERDICT.md`'s bottom-line table and candidate 18: cid 102, zero `upgrade`,
/// `link` or `.bin` keys as of the research) and the newer ODM chooser template's config
/// (candidate 19 / `sources/lofree-2026-web-driver-no-firmware.md`: cid 62, which already carries
/// a populated `upgrade` block of dead Pulsar sample links, so it is the file most likely to show
/// a live link first). Both sha256 values are recomputed from the archived snapshots at
/// `firmware/archive/other-devices/_web-driver-snapshots/web-snapshot-2026-09-15/www.lofree.tech/`.
pub const CONFIG_TARGETS: &[ConfigTarget] = &[
    ConfigTarget {
        url: "https://www.lofree.tech/cfg.json",
        description: "Lofree HYPACE driver config (cid 102)",
        recorded_sha256: "a6fe720f1668de619a71607467a1f6e963360eb186fd8c22993d0ee1290d8415",
    },
    ConfigTarget {
        url: "https://www.lofree.tech/home/cfg.json",
        description: "Lofree ODM chooser template config (cid 62)",
        recorded_sha256: "3b9c365258018a5fc6de07105d110c17ea076ffe1e53d00ca889896356e5acd2",
    },
];

/// One firmware directory path this module probes, named in `FIRMWARE-VERDICT.md` section 3c
/// (candidates 22 to 24): a path a Vite-template HYPACE app would fetch a `.bin` link's directory
/// from, for this mouse's own cid (102, the production `/mouse` driver) and the chooser's cid (62,
/// the `/home` template).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectoryTarget {
    /// Full URL to probe (trailing slash, matching how the verdict names each path).
    pub url: &'static str,
    /// What this path is, for display.
    pub description: &'static str,
}

/// The firmware directory paths named in `FIRMWARE-VERDICT.md` candidates 22 to 24, restricted to
/// this hardware's own cid (102) and the chooser's cid (62); candidates naming an unrelated cid
/// (66, 87) or the unrelated `/OE921/` keyboard family are not this hardware's own identity and
/// are not probed here.
pub const DIRECTORY_TARGETS: &[DirectoryTarget] = &[
    DirectoryTarget {
        url: "https://www.lofree.tech/mouse/bin/",
        description: "production HYPACE driver's own firmware directory (candidate 24)",
    },
    DirectoryTarget {
        url: "https://www.lofree.tech/home/bin/cid102_mid01/",
        description: "chooser template's firmware directory, this mouse's cid/mid 1 (candidate 23)",
    },
    DirectoryTarget {
        url: "https://www.lofree.tech/home/bin/cid102_mid02/",
        description: "chooser template's firmware directory, this mouse's cid/mid 2 (candidate 23)",
    },
    DirectoryTarget {
        url: "https://www.lofree.tech/keyboard/bin/cid102_mid01/",
        description: "keyboard deployment's firmware directory, this mouse's cid/mid 1 (candidate 22)",
    },
    DirectoryTarget {
        url: "https://www.lofree.tech/keyboard/bin/cid102_mid02/",
        description: "keyboard deployment's firmware directory, this mouse's cid/mid 2 (candidate 22)",
    },
    DirectoryTarget {
        url: "https://www.lofree.tech/keyboard/bin/cid62_mid01/",
        description: "keyboard deployment's firmware directory, the chooser's cid (candidate 22)",
    },
];

/// Bytes of context kept on each side of a matched needle in [`find_evidence`], bounding every
/// quoted snippet to a fixed, readable size regardless of how large the surrounding file is.
const EVIDENCE_CONTEXT_BYTES: usize = 80;
/// Upper bound on how many quoted snippets one [`evaluate_config`] call returns, so a
/// pathological file (many repeated matches) cannot produce an unbounded finding.
const MAX_EVIDENCE_SNIPPETS: usize = 6;

/// Hex-encoded sha256 of `bytes`.
fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            use core::fmt::Write as _;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

/// The nearest valid `char` boundary at or before `idx`, so a snippet never splits a multi-byte
/// UTF-8 sequence. `text.is_char_boundary` is `O(1)`; this walks at most 3 bytes back (the longest
/// UTF-8 sequence is 4 bytes).
fn char_boundary_floor(text: &str, mut idx: usize) -> usize {
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// The nearest valid `char` boundary at or after `idx`.
fn char_boundary_ceil(text: &str, mut idx: usize) -> usize {
    while idx < text.len() && !text.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

/// A bounded, char-boundary-safe window of `text` around byte offset `start`, trimmed of leading
/// and trailing whitespace.
fn snippet_around(text: &str, start: usize, needle_len: usize) -> String {
    let lo = char_boundary_floor(text, start.saturating_sub(EVIDENCE_CONTEXT_BYTES));
    let hi = char_boundary_ceil(
        text,
        (start + needle_len + EVIDENCE_CONTEXT_BYTES).min(text.len()),
    );
    text[lo..hi].trim().to_owned()
}

/// Scan `body` for the two markers the verdict names as decisive: an `"upgrade"` JSON key (the
/// block name every archived config uses for firmware links) and a `.bin` reference (the archive's
/// universal image extension). Returns a bounded, deduplicated list of quoted snippets, one per
/// match found, up to [`MAX_EVIDENCE_SNIPPETS`].
///
/// This is a literal substring scan over the bytes as lossy UTF-8, not a JSON parse: it quotes
/// exactly what is in the file rather than a reinterpretation of it, and it costs nothing when the
/// content carries neither marker (the overwhelmingly common case for every target here today).
/// The caller is responsible for bounding `body`'s size before calling this (`hyperpace-app` caps
/// every fetch); this function performs no I/O and allocates only the returned snippets.
fn find_evidence(body: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(body);
    let mut snippets = Vec::new();
    for needle in ["\"upgrade\"", ".bin"] {
        for (start, _) in text.match_indices(needle) {
            if snippets.len() >= MAX_EVIDENCE_SNIPPETS {
                return snippets;
            }
            let snippet = snippet_around(&text, start, needle.len());
            if !snippets.contains(&snippet) {
                snippets.push(snippet);
            }
        }
    }
    snippets
}

/// What [`evaluate_config`] found for one [`ConfigTarget`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFinding {
    /// The target this finding is for.
    pub url: &'static str,
    /// What this file is, for display.
    pub description: &'static str,
    /// sha256 recorded for this file at the time of the research.
    pub recorded_sha256: &'static str,
    /// sha256 of the bytes actually fetched.
    pub fetched_sha256: String,
    /// Whether `fetched_sha256` differs from `recorded_sha256`.
    pub changed: bool,
    /// Quoted snippets around an `"upgrade"` key or a `.bin` reference found in the fetched
    /// content. Only populated when `changed` is true: `/home/cfg.json`'s baseline already
    /// contains an `upgrade` block of dead links, so surfacing it again on every unchanged check
    /// would not be a finding, only noise.
    pub evidence: Vec<String>,
}

impl ConfigFinding {
    /// Whether this finding is worth a human's attention: the file changed at all.
    #[must_use]
    pub fn is_notable(&self) -> bool {
        self.changed
    }
}

impl fmt::Display for ConfigFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.changed {
            return write!(
                f,
                "{}: unchanged, sha256 matches the value recorded in the research",
                self.description
            );
        }
        write!(
            f,
            "{}: changed (sha256 now {}, recorded {}); ",
            self.description, self.fetched_sha256, self.recorded_sha256
        )?;
        if self.evidence.is_empty() {
            write!(
                f,
                "no upgrade block or .bin reference found in the new content. {EVIDENCE_DISCLAIMER}."
            )
        } else {
            write!(
                f,
                "found {} mention(s) of an upgrade block or .bin reference. {EVIDENCE_DISCLAIMER}.",
                self.evidence.len()
            )
        }
    }
}

/// Compare `body` (bytes already fetched from `target.url`) against `target`'s recorded baseline.
#[must_use]
pub fn evaluate_config(target: &ConfigTarget, body: &[u8]) -> ConfigFinding {
    let fetched_sha256 = hex_sha256(body);
    let changed = fetched_sha256 != target.recorded_sha256;
    let evidence = if changed {
        find_evidence(body)
    } else {
        Vec::new()
    };
    ConfigFinding {
        url: target.url,
        description: target.description,
        recorded_sha256: target.recorded_sha256,
        fetched_sha256,
        changed,
        evidence,
    }
}

/// What a directory probe observed, per `FIRMWARE-VERDICT.md`'s documented oracle: "on that host
/// an existing directory answers 403 while a missing path returns the single-page-app index, so
/// content, not status code, decides." A `200` alone is not proof of "missing": only a `200` whose
/// body matches the recorded SPA fallback hash is; a `200` with different content is itself a
/// finding, since something other than the SPA fallback is now being served at that path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryState {
    /// `200`, body matches [`RECORDED_SPA_INDEX_SHA256`]: this path does not exist yet, same as
    /// the verdict recorded.
    MissingAsRecorded,
    /// `403`: the verdict's documented signal that this path exists as a real directory
    /// (candidates 22 and 24: "existing dirs return 403").
    ExistsAsDirectory,
    /// `200`, but the body's sha256 does not match the recorded SPA fallback: something other
    /// than the single-page-app fallback is being served here now.
    ContentChanged {
        /// sha256 of the body actually fetched.
        fetched_sha256: String,
    },
    /// Any status other than the two the verdict documents. Not itself evidence of anything, only
    /// a shape this module does not otherwise recognize; reported rather than silently dropped.
    Unexpected {
        /// The HTTP status observed.
        status: u16,
    },
}

/// What [`evaluate_directory`] found for one [`DirectoryTarget`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryFinding {
    /// The target this finding is for.
    pub url: &'static str,
    /// What this path is, for display.
    pub description: &'static str,
    /// What was observed.
    pub state: DirectoryState,
}

impl DirectoryFinding {
    /// Whether this finding is worth a human's attention: anything but the recorded "still
    /// missing" shape.
    #[must_use]
    pub fn is_notable(&self) -> bool {
        !matches!(self.state, DirectoryState::MissingAsRecorded)
    }
}

impl fmt::Display for DirectoryFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.state {
            DirectoryState::MissingAsRecorded => write!(
                f,
                "{}: still missing, same single-page-app fallback the research recorded",
                self.description
            ),
            DirectoryState::ExistsAsDirectory => write!(
                f,
                "{}: now answers 403, the documented signal that a directory exists here where the \
                 research recorded none. {EVIDENCE_DISCLAIMER}.",
                self.description
            ),
            DirectoryState::ContentChanged { fetched_sha256 } => write!(
                f,
                "{}: now serves content other than the recorded single-page-app fallback (sha256 \
                 {fetched_sha256}, recorded {RECORDED_SPA_INDEX_SHA256}). {EVIDENCE_DISCLAIMER}.",
                self.description
            ),
            DirectoryState::Unexpected { status } => write!(
                f,
                "{}: returned status {status}, neither the documented 403 (exists) nor 200 \
                 (missing) shape; worth checking by hand.",
                self.description
            ),
        }
    }
}

/// Evaluate a directory probe: `status` and `body` are exactly what a caller received fetching
/// `target.url`.
#[must_use]
pub fn evaluate_directory(target: &DirectoryTarget, status: u16, body: &[u8]) -> DirectoryFinding {
    let state = if status == 403 {
        DirectoryState::ExistsAsDirectory
    } else if status == 200 {
        let fetched_sha256 = hex_sha256(body);
        if fetched_sha256 == RECORDED_SPA_INDEX_SHA256 {
            DirectoryState::MissingAsRecorded
        } else {
            DirectoryState::ContentChanged { fetched_sha256 }
        }
    } else {
        DirectoryState::Unexpected { status }
    };
    DirectoryFinding {
        url: target.url,
        description: target.description,
        state,
    }
}

/// Every finding from one watch pass, plus any target that could not be fetched at all.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WatchReport {
    /// One finding per [`CONFIG_TARGETS`] entry that was fetched successfully.
    pub configs: Vec<ConfigFinding>,
    /// One finding per [`DIRECTORY_TARGETS`] entry that was fetched successfully.
    pub directories: Vec<DirectoryFinding>,
    /// `(url, reason)` for every target a caller could not fetch at all (network error, timeout,
    /// or a response too large to evaluate); distinct from [`DirectoryState::Unexpected`], which
    /// is a successful fetch that returned an unrecognized status.
    pub fetch_errors: Vec<(String, String)>,
}

impl WatchReport {
    /// Whether anything in this report is worth a human's attention: a config change, a directory
    /// no longer matching the recorded "missing" shape, or a fetch failure.
    #[must_use]
    pub fn has_findings(&self) -> bool {
        self.configs.iter().any(ConfigFinding::is_notable)
            || self.directories.iter().any(DirectoryFinding::is_notable)
            || !self.fetch_errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// `ConfigTarget::recorded_sha256` is `&'static str` by design (every real target is a fixed
    /// const table entry); a test computing a hash at runtime needs to leak it to get that
    /// lifetime, which is fine for the lifetime of a test process.
    fn leak_hash(bytes: &[u8]) -> &'static str {
        Box::leak(hex_sha256(bytes).into_boxed_str())
    }

    #[test]
    fn an_unchanged_config_reports_no_evidence() {
        let target = ConfigTarget {
            url: "https://example.invalid/cfg.json",
            description: "test config",
            recorded_sha256: leak_hash(b"{}"),
        };
        let finding = evaluate_config(&target, b"{}");
        assert!(!finding.changed);
        assert!(finding.evidence.is_empty());
        assert!(!finding.is_notable());
        assert!(finding.to_string().contains("unchanged"));
    }

    #[test]
    fn a_changed_config_without_firmware_markers_reports_no_evidence() {
        let target = ConfigTarget {
            url: "https://example.invalid/cfg.json",
            description: "test config",
            recorded_sha256: leak_hash(b"{\"a\":1}"),
        };
        let finding = evaluate_config(&target, b"{\"a\":2}");
        assert!(finding.changed);
        assert!(finding.evidence.is_empty());
        assert!(finding.is_notable());
        let rendered = finding.to_string();
        assert!(rendered.contains("changed"));
        assert!(rendered.contains(EVIDENCE_DISCLAIMER));
    }

    #[test]
    fn an_appearing_upgrade_block_is_quoted_as_evidence() {
        let baseline = b"{\"upgrade\":{}}";
        let target = ConfigTarget {
            url: "https://example.invalid/cfg.json",
            description: "test config",
            recorded_sha256: leak_hash(baseline),
        };
        let live = br#"{"upgrade":{"device":{"version":"v2.18","link":"/hypace_mouse.bin"}}}"#;
        let finding = evaluate_config(&target, live);
        assert!(finding.changed);
        assert!(!finding.evidence.is_empty());
        assert!(finding.evidence.iter().any(|s| s.contains("\"upgrade\"")));
        assert!(finding.evidence.iter().any(|s| s.contains(".bin")));
    }

    #[test]
    fn evidence_is_bounded_and_deduplicated() {
        use core::fmt::Write as _;
        let mut live = String::from("{\"upgrade\":{");
        for i in 0..20 {
            let _ = write!(live, "\"{i}\":\"a.bin\",");
        }
        live.push_str("\"last\":true}}");
        let target = ConfigTarget {
            url: "https://example.invalid/cfg.json",
            description: "test config",
            recorded_sha256: leak_hash(b"{}"),
        };
        let finding = evaluate_config(&target, live.as_bytes());
        assert!(finding.evidence.len() <= MAX_EVIDENCE_SNIPPETS);
    }

    #[test]
    fn a_403_is_classified_as_an_existing_directory() {
        let target = DirectoryTarget {
            url: "https://example.invalid/mouse/bin/",
            description: "test directory",
        };
        let finding = evaluate_directory(&target, 403, b"Forbidden");
        assert_eq!(finding.state, DirectoryState::ExistsAsDirectory);
        assert!(finding.is_notable());
        assert!(finding.to_string().contains("403"));
    }

    // A 200 whose body hashes to RECORDED_SPA_INDEX_SHA256 (MissingAsRecorded) cannot be
    // constructed here without already knowing bytes that hash to it; that path is proven against
    // the real archived vendor snapshot in tests/watch.rs, which reads the exact 608 byte index
    // page this constant was recomputed from.

    #[test]
    fn a_200_not_matching_the_recorded_spa_index_is_content_changed() {
        let target = DirectoryTarget {
            url: "https://example.invalid/mouse/bin/",
            description: "test directory",
        };
        let finding = evaluate_directory(&target, 200, b"a firmware index, maybe");
        assert!(matches!(
            finding.state,
            DirectoryState::ContentChanged { .. }
        ));
        assert!(finding.is_notable());
        assert!(finding.to_string().contains("other than the recorded"));
    }

    #[test]
    fn an_unrecognized_status_is_reported_but_distinguished() {
        let target = DirectoryTarget {
            url: "https://example.invalid/mouse/bin/",
            description: "test directory",
        };
        let finding = evaluate_directory(&target, 500, b"oops");
        assert_eq!(finding.state, DirectoryState::Unexpected { status: 500 });
        assert!(finding.is_notable());
    }

    #[test]
    fn watch_report_has_findings_when_anything_is_notable() {
        let mut report = WatchReport::default();
        assert!(!report.has_findings());
        report
            .fetch_errors
            .push(("url".to_owned(), "timeout".to_owned()));
        assert!(report.has_findings());
    }

    #[test]
    fn config_and_directory_target_lists_are_well_formed() {
        for target in CONFIG_TARGETS {
            assert!(target.url.starts_with("https://"));
            assert_eq!(target.recorded_sha256.len(), 64);
            assert!(
                target
                    .recorded_sha256
                    .chars()
                    .all(|c| c.is_ascii_hexdigit())
            );
        }
        for target in DIRECTORY_TARGETS {
            assert!(target.url.starts_with("https://"));
            assert!(target.url.ends_with('/'));
        }
        assert_eq!(RECORDED_SPA_INDEX_SHA256.len(), 64);
    }
}
