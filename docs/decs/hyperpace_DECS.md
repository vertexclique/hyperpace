# Decision log: Hyperpace autonomous run

Plan: `docs/plans/hyperpace.md`. Mode: loop, started 2026-09-16 on the operator's instruction
"Run all phases back to back" and "Finish everything". In this mode I do not stop at phase gates;
every decision I take alone is recorded here for review.

Format: one row per decision, newest at the bottom.

| # | Phase | Decision | Why | Alternative not taken | Reversible? |
|---|---|---|---|---|---|
| 1 | 1 | Workspace edition 2024, resolver 3, lints `missing_docs`, `unsafe_code = forbid`, clippy all plus pedantic, `unwrap_used` and `expect_used` warned | Matches the doctrine's no-panic-on-production-path rule and the toolchain present (rustc 1.98) | Edition 2021 to match the store dependency's own edition; not needed, since editions compose per crate | Yes, one manifest |
| 2 | 1 | `Frame::decode` does not enforce the checksum; `Frame::checksum_valid` is separate | A device reply is useful even when its trailing byte is not what a request carries, and the research never proved the device checksums its responses | Reject frames with a bad checksum, which would drop valid replies if the firmware does not checksum | Yes |
| 3 | 1 | `declared_payload()` clamps the device supplied length byte to the real payload size | The length byte is device supplied and is known to understate the payload for at least one command; an unclamped index would panic | Trust the byte, which risks a panic on the device path | Yes |
| 4 | 1 | `Status::Other(u8)` keeps unknown status bytes verbatim instead of mapping them to an error | The firmware's full status set is unknown; discarding the value would lose evidence | Treat anything above 1 as a hard error | Yes |
| 5 | 1 | Test modules allow `clippy::unwrap_used` | The doctrine bans panics on production paths, and explicitly permits assertions in tests | Result-returning tests everywhere, which hides the failing line | Yes |
| 6 | 1 to 8 | The `Transport` trait is defined in `hyperpace-protocol`, not in `hyperpace-device` | It is pure data flow with no I/O, and placing it in the protocol crate lets the firmware crate depend on protocol alone, so device and firmware can be built in parallel | Keep it in the device crate and make firmware depend on device, which serializes the build and couples firmware to hidapi | Yes, a file move |
| 7 | 1 to 8 | I pre-created the four crate skeletons and registered every workspace member myself | Parallel coding agents would otherwise edit the root manifest at the same time and lose each other's edits; the skeletons contain documentation only, no stub logic | Let each agent add its own member entry | Yes |
| 8 | 1 to 8 | Implementation fanned out to coder agents on the fast model, staged protocol and store, then device and firmware, then app and UI, then packaging, then gate rounds | The doctrine reserves the reasoning tier for design and delegates code writing; the staging matches the dependency order so no agent waits on a peer's files | One sequential implementer, which would take far longer for the same result | Yes |
| 9 | 7 | Firmware tests parse the archived sibling images read-only and assert they are classified as foreign | It proves the parser and the identity gate against real vendor containers without any genuine package for this hardware | Synthetic fixtures only, which would not prove the gate rejects real foreign images | Yes |
| 10 | Integration | The Tauri app now points at the real UI build (`../../ui/build`) with a `beforeBuildCommand` that builds it, and the placeholder page under the app crate was deleted | The app shipped by the agents pointed at a placeholder, so it would have launched without the interface; this is a defect fix, not a preference | Keep the placeholder and copy the UI output into it at package time, which duplicates artefacts and drifts | Yes |
| 11 | Integration | Dependency policy extended with MPL-2.0, Unlicense and BSL-1.0 after naming every crate that carries them | The nine rejected crates are all transitive (webview CSS stack, the Linux tray backend, a hashing crate); all three families permit linking and redistribution in a binary, and refusing them would mean dropping the webview or the tray | Vendor or replace those dependencies, which is not possible for the webview stack | Yes |
| 12 | Integration | Two RUSTSEC advisories accepted by id with written reasons: fxhash (RUSTSEC-2025-0057) and proc-macro-error (RUSTSEC-2024-0370) | Both are unmaintained notices with no reported vulnerability and no safe upgrade, and both are baked into third-party manifests with no feature to disable them; they are recorded in `deny.toml` rather than silently suppressed | Leave the gate red, which would make the gate meaningless as a signal | Yes, on the next dependency upgrade |
| 13 | Integration | Makefile `clean` no longer deletes `dist/` | `dist/` is source-controlled packaging input, not build output; the target now removes `target` and the UI build output instead | Rename the packaging directory to avoid the clash, which contradicts the operator's instruction that packaging lives under `dist` | Yes |

## Standing constraints during this run

- The operator's mouse is never opened. All testing runs against the simulator.
- No claim of hardware verification for anything, firmware flashing above all.
- Firmware install and rollback are implemented but have no genuine package to act on.
- No vendor branding in any product surface; vendor names appear only in `docs/research` and
  `docs/papers` as citations.
- No em-dash or en-dash anywhere.
