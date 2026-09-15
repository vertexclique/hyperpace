# Research: Hyperpace firmware update and platform stack

Date: 2026-09-15. Gate: phase-gate (runs to completion, stops at JUDGEMENT.md before planning).

## Questions

1. Firmware: where do official firmware images and the update procedure for the target mouse
   (VID 0x3554, PID 0xFB16 receiver / 0xFB14 wired; nRF52833 mouse MCU, CX52650N and CH32V305
   receivers) live, and what is the flashing protocol (vendor commands 13 EnterUsbUpdateMode,
   17 EnterMTKMode, HID report ID 9, 48 bytes)?
2. Platform: what does current Tauri 2 offer for tray/background running, notifications,
   autostart, single instance, self-update, and AppImage + Windows + macOS bundling?
3. HID: how to reach the vendor HID collection from Rust on Linux, Windows and macOS, with
   hotplug, correct report lengths and permissions?

4. Version archive and rollback (added 2026-09-15 by the operator): obtain every released firmware
   version for the mouse and for every receiver variant (1 kHz, 2 kHz, 4 kHz, 8 kHz), store them
   durably in `firmware/archive/` with provenance, and determine per chip whether the bootloader
   accepts an older image (anti-rollback checks, signed version fields) and how a failed
   downgrade is recovered. Hyperpace must bundle all versions and offer revert.

What settles question 1: an official firmware image plus a recoverable update protocol (vendor
updater code, a web updater, or a documented chip-level path with a recovery method). A
negative result is recorded as such; nothing is flashed on the strength of a guess.

Stop condition per stream: five consecutive differently-angled searches add nothing new.

## Streams

| Stream | Scope | Search log |
|---|---|---|
| vendor | official firmware and updaters, sibling drivers on the same ODM template with an `upgrade` block, static analysis of downloaded packages | `sources/_searchlog-vendor.md` |
| chips | Nordic nRF52833 DFU paths (incl. DFU over HID), WCH CH32V305 ISP, CX52650N identity, meaning of the unused update commands, brick and recovery | `sources/_searchlog-chips.md` |
| tauri | Tauri 2 versions, embedding, tray, plugins, bundling, signing, Svelte 5 integration, WebKitGTK pitfalls | `sources/_searchlog-tauri.md` |
| hid | hidapi and alternatives, Windows report length and collections, macOS permissions, Linux udev, hotplug, concurrency | `sources/_searchlog-hid.md` |

## Rules for this run

- Operator authorized downloading vendor updater and firmware packages for static analysis
  only; nothing downloaded is executed. Binaries live in the session scratch directory, never
  in this folder; each is recorded by URL, size and sha256 in `sources/`.
- No interaction with the connected mouse during research, or at any later stage: it is the
  operator's daily device.
- Verification strategy (operator, 2026-09-15): code-derived only. No spare unit and no traffic
  capture. The settings protocol is proven by differential tests against the vendor's own
  driver code. Readable vendor updater logic is therefore the only oracle for the flashing
  protocol.
- Feature availability (operator, 2026-09-15, "Nothing must be blocked"): firmware install
  and rollback ship fully enabled, with no lock or unverified banner. Only guards that prevent
  a certain brick remain: checksum match, image matches the target component, sufficient
  battery, do-not-unplug warning.
- Vendor names appear here only as citations; the Hyperpace product carries no vendor branding.

## Local tooling (checked 2026-09-15)

- Sealed JavaScript execution for the differential oracle: bubblewrap 0.12.0 (`/usr/bin/bwrap`)
  wrapping node v26.8.1, with network, filesystem and environment unshared except the harness
  directory. No deno or bun installed. systemd-run and unshare are also available.
- Unpacking: bsdtar, strings, file, objdump. No 7z or innoextract installed (the vendor stream
  may fetch the official 7zz into scratch).
- Rust 1.98.0 with Linux, Windows (gnu, msvc), macOS and WASM targets installed; webkit2gtk-4.1
  2.52.6 and libudev development files present; cargo-tauri CLI not installed.

## Layout

- `sources/`: one file per load-bearing source (URL, retrieval date, verbatim excerpts).
- `summaries/`: `SUMMARY-<key>.md` per source (claim, method, result, evidence tier,
  performance, correctness, relevance).
- `JUDGEMENT.md`: ranked verdict for the planning stage.

Prior context: `docs/research/lofree-hypace-protocol.md` (configuration protocol read from the
vendor web driver).

## What was gathered

Filled in when the streams complete.
