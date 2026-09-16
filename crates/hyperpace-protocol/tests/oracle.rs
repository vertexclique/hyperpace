//! Differential oracle: `hyperpace-protocol` against the vendor's own JavaScript.
//!
//! `docs/plans/hyperpace.md` phase 1 gate and section 6. Until this test existed, the codec was
//! checked only against golden vectors hand-derived from `docs/research/mouse-protocol-v2.md`.
//! This test instead loads `tests/vectors/oracle.json`, a fixture of frames captured by running
//! the vendor's own OLD (`app.js`, cid 102) and NEW (`home/index-BTVblIUr.js`, cid 62) driver
//! code, sealed under bubblewrap with a stub `navigator.hid` (`scripts/oracle.sh`, regenerated
//! with `make oracle`), and asserts this crate reproduces every byte.
//!
//! The fixture is committed, so this test runs with no sandbox and no JavaScript at hand; only
//! `make oracle` touches either. A second test below re-hashes the two vendor bundles this
//! fixture was generated from and fails loudly if they have drifted, so a stale fixture cannot
//! pass silently.

// Tests may assert (PRINCIPLES.md 3, and frame.rs's own test module does the same): a fixture
// bug or an unrecognized shape here should fail the test loudly, which a panic does exactly as
// well as a `Result`, with far less ceremony for a file whose only reader is this test binary.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines,
    clippy::cast_possible_truncation
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use hyperpace_protocol::{
    ButtonAction, Command, DeviceIdentity, Keystroke, LinkType, MacroEvent, MacroEventKind,
    MacroSlot, Modifier, REPORT_ID, ReceiverLight, Shadow, config_file, encoding, model, offset,
    request,
};

#[derive(Debug, Deserialize)]
struct Fixture {
    meta: Meta,
    operations: Vec<Operation>,
}

#[derive(Debug, Deserialize)]
struct Meta {
    bundles: BTreeMap<String, BundleInfo>,
    operation_count: usize,
}

#[derive(Debug, Deserialize)]
struct BundleInfo {
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Operation {
    name: String,
    category: String,
    input: Value,
    address: u16,
    #[serde(default)]
    frames: Vec<CapturedFrame>,
    #[serde(default)]
    vendor_clamped_bytes: Option<Vec<u8>>,
    #[serde(default)]
    checksum_result: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CapturedFrame {
    report_id: u8,
    bytes: Vec<u8>,
}

fn vendor_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../firmware/archive/other-devices/_web-driver-snapshots")
        .join("web-snapshot-2026-09-15/www.lofree.tech")
}

fn load_fixture() -> Fixture {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/oracle.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes).expect("tests/vectors/oracle.json must be valid fixture JSON")
}

/// The fixture is only ever honest about the vendor code it claims to have run if the two
/// bundles it names still hash to what it recorded. `firmware/archive` is read-only evidence
/// (never edited by this crate's own tooling), so a mismatch here means either the archive
/// changed underneath the fixture or the fixture is stale: regenerate it with `make oracle`.
#[test]
fn fixture_bundle_hashes_match_the_current_vendor_snapshot() {
    let fixture = load_fixture();
    for (label, info) in &fixture.meta.bundles {
        let path = vendor_dir().join(&info.path);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let got = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(
            got,
            info.sha256,
            "{label} bundle ({}) sha256 no longer matches tests/vectors/oracle.json; \
             re-run `make oracle` to regenerate the fixture",
            path.display()
        );
    }
}

fn be16(bytes: &[u8]) -> u16 {
    u16::from_be_bytes([bytes[0], bytes[1]])
}

/// The one frame in `frames` whose command byte and address match, if any. Every operation in
/// the matrix targets exactly one address, so this is enough to pick the frame under test out of
/// a capture that may also carry an online-gate probe (command 3) or, for a profile switch, a
/// full flash re-read (section 10.2) alongside it.
fn find_frame(frames: &[CapturedFrame], command: u8, address: u16) -> Option<&[u8]> {
    frames
        .iter()
        .find(|f| f.bytes[0] == command && be16(&f.bytes[2..4]) == address)
        .map(|f| f.bytes.as_slice())
}

/// Reassembles a multi-frame `WriteFlashData` write (section 4.10's `st`/`Qe` chunker) back into
/// the logical byte sequence a codec-level encoder produces, by concatenating each chunk's
/// declared payload in address order. Used for keystroke and macro slots, which do not fit one
/// 10-byte frame.
fn reassemble_write(frames: &[CapturedFrame], base_address: u16) -> Vec<u8> {
    let mut chunks: Vec<&CapturedFrame> = frames.iter().filter(|f| f.bytes[0] == 7).collect();
    chunks.sort_by_key(|f| be16(&f.bytes[2..4]));
    let mut out = Vec::new();
    let mut expect_addr = base_address;
    for f in chunks {
        let addr = be16(&f.bytes[2..4]);
        if addr != expect_addr {
            continue; // not part of this write's contiguous chunk sequence
        }
        let len = usize::from(f.bytes[4]).min(10);
        out.extend_from_slice(&f.bytes[5..5 + len]);
        expect_addr = expect_addr.wrapping_add(len as u16);
    }
    out
}

fn parse_rgb(s: &str) -> (u8, u8, u8) {
    let inner = s
        .strip_prefix("rgb(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or_else(|| panic!("not an rgb(...) string: {s}"));
    let mut parts = inner.split(',').map(|p| p.trim().parse::<u8>().unwrap());
    (
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
    )
}

/// The keymap entries (section 8.6) this fixture's shortcut chords actually use. Not a copy of
/// the vendor's 104-entry keymap: just enough to decode the chords `scripts/oracle/harness/generate.mjs`
/// built by display text, the same way `Set_MS_ShortcutKey`'s `textToHID` does.
fn text_to_modifier(text: &str) -> Option<Modifier> {
    match text {
        "LCtrl" => Some(Modifier::LeftCtrl),
        "LShift" => Some(Modifier::LeftShift),
        "LAlt" => Some(Modifier::LeftAlt),
        "LWin" => Some(Modifier::LeftWin),
        "RCtrl" => Some(Modifier::RightCtrl),
        "RShift" => Some(Modifier::RightShift),
        "RAlt" => Some(Modifier::RightAlt),
        "RWin" => Some(Modifier::RightWin),
        _ => None,
    }
}

fn text_to_key(text: &str) -> Option<u8> {
    match text {
        "A" => Some(4),
        "Z" => Some(29),
        "Enter" => Some(40),
        "F1" => Some(58),
        "Space" => Some(44),
        _ => None,
    }
}

fn record_mismatch(failures: &mut Vec<String>, name: &str, expected: &[u8], actual: &[u8]) {
    if expected != actual {
        failures.push(format!(
            "{name}:\n  rust:   {expected:02x?}\n  vendor: {actual:02x?}"
        ));
    }
}

fn record_missing(failures: &mut Vec<String>, name: &str, command: u8, address: u16) {
    failures.push(format!(
        "{name}: no vendor frame found for command {command} at address {address}"
    ));
}

fn model_table(name: &str) -> &'static model::ModelTable {
    match name {
        "old" => &model::CID_102_MID_1,
        "new" => &model::CID_62_MID_1,
        other => panic!("unknown model table label: {other}"),
    }
}

#[test]
fn hyperpace_protocol_matches_the_vendor_byte_for_byte() {
    let fixture = load_fixture();
    assert_eq!(
        fixture.operations.len(),
        fixture.meta.operation_count,
        "fixture header's operation_count disagrees with the operations it carries"
    );

    let mut failures: Vec<String> = Vec::new();
    let mut compared = 0usize;
    let mut per_category: BTreeMap<String, usize> = BTreeMap::new();

    for op in &fixture.operations {
        *per_category.entry(op.category.clone()).or_insert(0) += 1;
        compared += 1;

        // Every capture rides HID report id 8 (section 4.2): a hardware-level fact the sandbox's
        // stub `navigator.hid` cannot fake, since it is the vendor bundle's own module constant.
        for frame in &op.frames {
            assert_eq!(
                frame.report_id, REPORT_ID,
                "{}: captured frame used report id {}, not {REPORT_ID}",
                op.name, frame.report_id
            );
        }

        match op.category.as_str() {
            "command_only" => {
                let cmd = op.input["command"].as_u64().unwrap() as u8;
                let command = Command::from_byte(cmd).expect("known command byte");
                let expected = command.request().encode();
                match find_frame(&op.frames, cmd, 0) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, cmd, 0),
                }
            }

            "scalar" | "scalar_computed" => {
                let value = if let Some(hz) = op.input.get("hz").and_then(Value::as_u64) {
                    encoding::polling_to_byte(u16::try_from(hz).unwrap())
                        .expect("documented polling rate")
                } else if let Some(level) = op.input.get("level").and_then(Value::as_u64) {
                    encoding::indicator_brightness_to_byte(u8::try_from(level).unwrap())
                } else {
                    u8::try_from(op.input["value"].as_u64().unwrap()).unwrap()
                };
                let expected = request::set_scalar(op.address, value).encode();
                match find_frame(&op.frames, 7, op.address) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, 7, op.address),
                }
            }

            "dpi_value" => {
                // `input.stage` is carried in the fixture for readability only; `op.address`
                // (12 + 4 * stage) already pins the frame under test.
                let dpi = u32::try_from(op.input["dpi"].as_u64().unwrap()).unwrap();
                let table = model_table(op.input["table"].as_str().unwrap());
                let record = encoding::dpi_to_bytes(dpi, table)
                    .unwrap_or_else(|e| panic!("{}: dpi_to_bytes({dpi}) failed: {e:?}", op.name));
                let expected = request::write_flash(op.address, &record).unwrap().encode();
                match find_frame(&op.frames, 7, op.address) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, 7, op.address),
                }
            }

            "dpi_color" => {
                let (r, g, b) = parse_rgb(op.input["color"].as_str().unwrap());
                let record = [r, g, b, encoding::struct_check(&[r, g, b])];
                let expected = request::write_flash(op.address, &record).unwrap().encode();
                match find_frame(&op.frames, 7, op.address) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, 7, op.address),
                }
            }

            "button" => {
                let kind = u8::try_from(op.input["kind"].as_u64().unwrap()).unwrap();
                let param = u16::try_from(op.input["param"].as_u64().unwrap()).unwrap();
                let hi = (param >> 8) as u8;
                let lo = (param & 0xff) as u8;
                // Round-trip through decode: every (kind, param) pair decodes to something
                // (a named variant or `Unknown`) whose re-encode carries the same kind and
                // param, so this reconstructs the expected record without special-casing which
                // kind byte the fixture happens to carry.
                let record = ButtonAction::decode(&[kind, hi, lo, 0]).encode();
                let expected = request::write_flash(op.address, &record).unwrap().encode();
                match find_frame(&op.frames, 7, op.address) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, 7, op.address),
                }
            }

            "media" => {
                let usage = u16::try_from(op.input["usage"].as_u64().unwrap()).unwrap();
                let keystroke = Keystroke {
                    modifiers: vec![],
                    key: None,
                    media: Some(usage),
                };
                let expected = keystroke.encode();
                let actual = reassemble_write(&op.frames, op.address);
                record_mismatch(&mut failures, &op.name, &expected, &actual);
            }

            "shortcut" => {
                let chord: Vec<String> = serde_json::from_value(op.input["chord"].clone()).unwrap();
                let mut modifiers = Vec::new();
                let mut key = None;
                for text in &chord {
                    if let Some(m) = text_to_modifier(text) {
                        modifiers.push(m);
                    } else if let Some(k) = text_to_key(text) {
                        key = Some(k);
                    } else {
                        panic!("{}: no keymap mapping for {text:?}", op.name);
                    }
                }
                let keystroke = Keystroke {
                    modifiers,
                    key,
                    media: None,
                };
                let expected = keystroke.encode();
                let actual = reassemble_write(&op.frames, op.address);
                record_mismatch(&mut failures, &op.name, &expected, &actual);
            }

            "macro_full" => {
                let name = op.input["name"].as_str().unwrap().to_owned();
                let events = op.input["events"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|e| {
                        let press = e["press"].as_bool().unwrap();
                        let kind = u8::try_from(e["kind"].as_u64().unwrap()).unwrap();
                        let value = u16::try_from(e["value"].as_u64().unwrap()).unwrap();
                        let delay_ms = u16::try_from(e["delay"].as_u64().unwrap()).unwrap();
                        let kind = match kind {
                            0 => MacroEventKind::Modifier,
                            1 => MacroEventKind::Key,
                            2 => MacroEventKind::Consumer,
                            4 => MacroEventKind::Mouse,
                            other => MacroEventKind::Other(other),
                        };
                        MacroEvent {
                            press,
                            kind,
                            value,
                            delay_ms,
                        }
                    })
                    .collect();
                let slot = MacroSlot { name, events };
                let expected = slot
                    .encode()
                    .expect("valid macro under the fixture's own limits");
                let actual = reassemble_write(&op.frames, op.address);
                record_mismatch(&mut failures, &op.name, &expected, &actual);
            }

            "flash_read" => {
                let address = u16::try_from(op.input["address"].as_u64().unwrap()).unwrap();
                let len = u8::try_from(op.input["len"].as_u64().unwrap()).unwrap();
                let expected = request::read_flash(address, len).encode();
                match find_frame(&op.frames, 8, address) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, 8, address),
                }
            }

            "long_range" => {
                let on = op.input["on"].as_bool().unwrap();
                let expected = request::set_long_range(on).encode();
                match find_frame(&op.frames, 22, 0) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, 22, 0),
                }
            }

            "profile" => {
                let index = u8::try_from(op.input["index"].as_u64().unwrap()).unwrap();
                let expected = request::set_profile(index).encode();
                match find_frame(&op.frames, 15, 0) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, 15, 0),
                }
            }

            "receiver_light" => {
                let mode = u8::try_from(op.input["mode"].as_u64().unwrap()).unwrap();
                let (r, g, b) = parse_rgb(op.input["color"].as_str().unwrap());
                let speed = u8::try_from(op.input["speed"].as_u64().unwrap()).unwrap();
                let brightness = u8::try_from(op.input["brightness"].as_u64().unwrap()).unwrap();
                let time = u8::try_from(op.input["time"].as_u64().unwrap()).unwrap();
                let light = ReceiverLight {
                    mode,
                    color: (r, g, b),
                    speed,
                    brightness,
                    time,
                };
                let expected = request::receiver_light(&light).encode();
                match find_frame(&op.frames, 24, 0) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, 24, 0),
                }
            }

            "lighting_struct" => {
                let mode = u8::try_from(op.input["mode"].as_u64().unwrap()).unwrap();
                let (r, g, b) = parse_rgb(op.input["color"].as_str().unwrap());
                let speed = u8::try_from(op.input["speed"].as_u64().unwrap()).unwrap();
                let brightness = u8::try_from(op.input["brightness"].as_u64().unwrap()).unwrap();
                let data = [mode, r, g, b, speed, brightness];
                let record = [
                    mode,
                    r,
                    g,
                    b,
                    speed,
                    brightness,
                    encoding::struct_check(&data),
                ];
                let expected = request::write_flash(160, &record).unwrap().encode();
                match find_frame(&op.frames, 7, 160) {
                    Some(actual) => record_mismatch(&mut failures, &op.name, &expected, actual),
                    None => record_missing(&mut failures, &op.name, 7, 160),
                }
            }

            "config_import_clamp" => {
                // Section 11.2: the vendor clamps the current DPI stage to `count - 1` but
                // writes its check byte from the *count*, not the clamped stage. The fixture's
                // stage count sits exactly at the vendor UI's own ceiling, so that clamp is a
                // no-op and both sides agree on it; only the current-stage clamp fires, isolating
                // the check-byte bug from Hyperpace's separate choice to clamp the count itself
                // against the memory map's 8-slot capacity rather than the vendor's per-model
                // maxDpiCount.
                let vendor = op
                    .vendor_clamped_bytes
                    .as_ref()
                    .expect("config_import_clamp_bug carries vendorClampedBytes");
                let raw_stage_count =
                    u8::try_from(op.input["rawStageCount"].as_u64().unwrap()).unwrap();
                let raw_current_stage =
                    u8::try_from(op.input["rawCurrentStage"].as_u64().unwrap()).unwrap();

                // Build a minimal file the way Hyperpace's own `config_file::import` expects it.
                let mut shadow = Shadow::new();
                shadow.apply_read(
                    offset::DPI_STAGE_COUNT,
                    &encoding::scalar_pair(raw_stage_count),
                );
                shadow.apply_read(
                    offset::CURRENT_DPI,
                    &encoding::scalar_pair(raw_current_stage),
                );
                let identity = DeviceIdentity {
                    cid: 62,
                    mid: 1,
                    link: LinkType::Wireless1k,
                };
                let file = config_file::export(&shadow, &identity, "3950");
                let imported =
                    config_file::import(&file, &model::CID_62_MID_1).expect("well-formed file");

                let rust_count = imported.shadow.scalar(offset::DPI_STAGE_COUNT);
                let rust_count_check = imported.shadow.scalar(offset::DPI_STAGE_COUNT + 1);
                let rust_stage = imported.shadow.scalar(offset::CURRENT_DPI);
                let rust_stage_check = imported.shadow.scalar(offset::CURRENT_DPI + 1);

                // The clamped values themselves agree (both are correctness fixes to the same
                // out-of-range input).
                if [vendor[0], vendor[2]] != [rust_count, rust_stage] {
                    failures.push(format!(
                        "{}: clamped count/stage disagree, vendor {:?} vs rust {:?}",
                        op.name,
                        [vendor[0], vendor[2]],
                        [rust_count, rust_stage]
                    ));
                }
                // Hyperpace's own check bytes are each correctly paired with their own scalar.
                if rust_count_check != encoding::scalar_pair(rust_count)[1]
                    || rust_stage_check != encoding::scalar_pair(rust_stage)[1]
                {
                    failures.push(format!(
                        "{}: hyperpace's own check bytes are not correctly paired",
                        op.name
                    ));
                }
                // The vendor bug, confirmed present: its stage check byte is NOT correctly
                // paired with the clamped stage (it reused the count's complement instead), and
                // differs from what Hyperpace writes for the same clamped stage.
                let vendor_stage_check = vendor[3];
                let correct_stage_check = encoding::scalar_pair(vendor[2])[1];
                if vendor_stage_check == correct_stage_check {
                    failures.push(format!(
                        "{}: vendor's clamp bug did not reproduce (check byte was correctly paired); \
                         the oracle input no longer exercises section 11.2's bug",
                        op.name
                    ));
                }
                if vendor_stage_check == rust_stage_check {
                    failures.push(format!(
                        "{}: expected the vendor's buggy check byte to differ from Hyperpace's \
                         correct one; they matched, which would mean the bug is no longer present",
                        op.name
                    ));
                }
                // Else: expected. This is the deviation the task requires we assert explicitly.
            }

            "checksum_fn" => {
                let head: Vec<u8> = serde_json::from_value(op.input["head"].clone()).unwrap();
                let sum = head.iter().fold(0u8, |acc, b| acc.wrapping_add(*b));
                let expected = i64::from(0x55u8) - i64::from(sum);
                let actual = op
                    .checksum_result
                    .expect("checksum_fn carries checksumResult");
                if expected != actual {
                    failures.push(format!(
                        "{}: checksum mismatch, rust {expected} vs vendor {actual}",
                        op.name
                    ));
                }
            }

            other => panic!("{}: unknown fixture category {other:?}", op.name),
        }
    }

    eprintln!(
        "oracle: compared {compared} operations across {} categories, {} bundles ({})",
        per_category.len(),
        fixture.meta.bundles.len(),
        fixture
            .meta
            .bundles
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    for (category, count) in &per_category {
        eprintln!("  {category}: {count}");
    }

    assert!(
        failures.is_empty(),
        "{} of {compared} oracle operations mismatched:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
