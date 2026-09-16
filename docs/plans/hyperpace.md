# Plan: Hyperpace

> Status: PLAN ONLY
> Source research: docs/papers/hyperpace-firmware-platform/JUDGEMENT.md
> No product code until this plan is approved.

## 0. TL;DR

- Build Hyperpace, a native desktop app (Rust core, Tauri 3.0 alpha shell, SvelteKit static SPA
  on Svelte 5 and TypeScript) that configures the operator's mouse over the vendor HID channel
  documented byte-exact in `docs/research/mouse-protocol-v2.md` (401 facts, 304 confirmed by two
  independent verifiers, 96 corrected, 1 refuted). One owner thread per device, dark UI, original
  SVG artwork, zero vendor branding in any product surface.
- Honest gap: no genuine firmware for this hardware exists on any channel reached (37 candidates,
  zero hits, `FIRMWARE-VERDICT.md`). The firmware engine is built to the verified spec
  (`docs/research/firmware-update-spec.md`) and ships enabled, but the archive is empty, so the
  firmware screen says "no firmware available for this device yet" and rollback stays impossible
  until complete vendor-built packages are acquired. Flashing is code-derived and has never been
  observed on hardware; it is never described as verified. Three acquisition routes are built in.
- Scale: nothing here is large, so the bounds that matter are latency and idle cost. The settings
  shadow is 16 KB read in 10-byte frames and written in 10-byte frames; firmware images stream to
  the device in 32-byte packets straight from file-backed embedded bytes and are never buffered
  whole; the tray destroys its webview when hidden and polls battery every 5 s.

## 1. Problem and context

The mouse ships only a browser configurator, reachable at two vendor deployments. The operator
wants a native app that runs in the background, shows battery clearly, warns before the battery
dies, configures every setting the vendor app can, and can install and roll back firmware offline.

Four constraints make this non-trivial, all established in the research:

1. **No hardware testing.** The operator's mouse is their daily device and must never be opened by
   me (memory rule, restated 2026-09-16). Correctness therefore comes from the vendor's own code:
   differential tests of the Rust codec against the vendor JavaScript, run sealed under bubblewrap
   (no network, filesystem or environment), plus a device simulator. The app itself may read state
   from the real mouse when the operator launches it; writes and flashing to the real device are
   not authorized yet.
2. **The vendor transaction layer is unsafe to copy.** It accepts a 3-byte echo as a match, treats
   an error status reply as success, does not serialize requests, and lets an unsolicited device
   push consume a retry and duplicate a write (`docs/research/mouse-protocol-v2.md`). Hyperpace
   implements a corrected layer rather than porting that behavior.
3. **Two vendor model tables disagree.** The production mouse page ships cid 102 (5 buttons,
   maxDpi 40000, two-range DPI table); the newer chooser ships cid 62 (6 buttons, 32000,
   single-range). The same DPI value encodes to different bytes under each, so the model table
   must be selected at runtime from the cid and mid the device reports
   (`sources/local-2026-vendor-page-deployments.md`).
4. **Firmware is unobtainable today and rollback cannot be manufactured.** No implementation
   synthesizes the prepare packet; all copy it from a vendor-built header, and the check value for
   this mouse's MCU family matches no tested CRC scheme. A dump, a raw payload or a rebuilt image
   is unflashable, and no read-back command exists to snapshot what is installed
   (`FIRMWARE-VERDICT.md`). Sibling images from other brands carry foreign product ids and would
   mis-target the hardware, so they are reference material only.

## 2. Approach

**Crate layout.** Five Rust crates plus the UI, each with one job:

| Crate | Responsibility | Notes |
|---|---|---|
| `hyperpace-protocol` | pure codec: frame build and parse, checksum, command table, memory map, all value encodings, model tables | no I/O, no async, fully testable; the differential harness targets this crate |
| `hyperpace-device` | hidapi owner thread per device, hotplug watcher, request and response matching, push dispatch, reconnect | one thread owns one device; the rest of the app talks over channels |
| `hyperpace-firmware` | image container parser, identity gate, update state machine, guards | operates on any reader; never opens a device itself |
| `hyperpace-store` | embedded defradb.rs node, storage only, under `$HOME/.hyperpace` | `Persistence::Persistent`, `TransportConfig::None`, no P2P |
| `hyperpace-app` | Tauri 3.0 alpha: commands, channels, tray, notifications, autostart, single instance | thin; holds no protocol knowledge |
| `ui/` | SvelteKit static SPA, dark theme, original SVG mouse and receiver art | no vendor artwork, no vendor names |

**Device layer.** One owner thread per device interleaves queued writes with short `read_timeout`
slices, so writes are serialized by construction and cancellation is cooperative. Requests are
matched by command byte, and device pushes are dispatched instead of being mistaken for replies.
Platform specifics from the research: on Windows select the collection by usage page 0xFF02 and
usage 0x0002 (report-8 writes are 17 bytes on this hardware, derived from its own descriptors); on
macOS open non-exclusively and keep one long-lived HID thread; on Linux ship a udev rule and show
the exact command when access is refused. Hotplug comes from a separate watcher started before
enumeration, since hidapi has none.

**Storage.** `defradb.rs` (git dependency, workspace 0.5.0, `Apache-2.0 OR MIT`, rust-version
1.91) embedded storage-only, with default features off so libp2p and the WASM runtime are not
pulled in. Everything lives under `$HOME/.hyperpace` on every OS: macro library, profile
snapshots, settings, the firmware archive index and the event log.

**Packaging.** All packaging inputs and outputs live under `dist/`: `dist/deb`, `dist/rpm`,
`dist/appimage`, `dist/arch` (PKGBUILD), `dist/windows` (NSIS), `dist/macos`. Native Linux
packages are the primary channel, AppImage is built and verified on this machine, and the Arch
package is first-class because that is what the operator runs. Builds are unsigned for now.

**Firmware.** The engine implements the verified update procedure end to end, with four guards
that only ever stop a flash that would certainly brick: checksum match, image identity matching
the target component, sufficient battery, and a do-not-unplug warning. Three acquisition routes
ship: a drafted vendor request the operator sends, a watcher that checks the vendor config files
and firmware directory for packages appearing, and an import path that accepts an
operator-supplied package only when its identity markers match this hardware.

## 3. Features: correctness, honesty, scale

| Feature | Math correct? | Honest? | Scales to target? | Action |
|---|---|---|---|---|
| Frame codec and checksum | ok (verified against both vendor drivers) | ok | ok | build in phase 1 |
| Settings memory map and encodings | ok, per model table | ok | ok (16 KB shadow, 10-byte frames) | build in phase 1 |
| DPI encode and decode | caveat: differs per model table | ok if the table is runtime-selected | ok | model table drives encoding, never a constant |
| Buttons, keystrokes, media keys | ok | ok | ok | phase 4 |
| Macros | ok | ok | ok (70 events, 384-byte slots) | phase 5 |
| Profiles | ok | ok | ok | phase 4 |
| Battery percentage | caveat: the vendor smooths a voltage curve | ok if labelled as an estimate | ok | show the device value, mark smoothing as an estimate |
| Low-battery warning | ok | ok | ok | phase 3 |
| Config export and import (.bin) | ok, and one vendor bug is fixed | ok | ok | phase 6, note the deviation |
| Receiver light, pairing, factory reset | ok | ok | ok | phase 6 |
| Firmware install | caveat: code-derived, never run on hardware | ok only if never called verified | ok (32-byte streamed packets) | phase 7, honest labelling |
| Firmware rollback | no: needs two or more vendor packages | ok if the empty state says so | n/a | phase 7 ships the machinery, not the capability |
| Firmware archive | no: zero images exist for this hardware | ok | ok | empty state says "no firmware available for this device yet" |
| Real-device writes | not authorized yet | ok | n/a | simulator only until the operator authorizes |

## 4. Phases

De-risk first. No phase depends on a later one. Each phase gates the next, and I ask before
starting each.

- **Phase 0 (de-risk spikes): SKIPPED by operator decision, 2026-09-16.** The four experiments are
  not run up front; each becomes a risk carried by the phase that first needs it, and each keeps
  its fallback:
  (a) `defradb.rs` embedded storage-only build, first needed in phase 5 (macro library); fallback
  is a plain file store, and I ask before switching.
  (b) Tauri 3.0 alpha plus its alpha plugins, first needed in phase 3; fallback is 2.11 stable,
  and I ask before switching.
  (c) AppImage launching on current Mesa, first needed in phase 8; fallback is native packages
  only.
  (d) The sandboxed vendor-code oracle, needed immediately in phase 1; if it cannot run sealed,
  verification falls back to committed golden vectors alone and I surface that at once, since it
  weakens every correctness claim in this plan.
- **Phase 1.** `hyperpace-protocol` plus the differential harness. Gates: byte-identical frames
  against the vendor implementation for every command, every setting, both model tables and the
  documented edge cases.
- **Phase 2.** `hyperpace-device` plus the device simulator (implementing the vendor's quirks:
  the battery reply length byte, error-status replies, unsolicited pushes). Gates: full connect
  sequence, reconnect, hotplug, and no path that can write to a real device.
- **Phase 3.** App shell: window, dark UI, tray with battery, low-battery notification, autostart,
  single instance. Gates: it runs on the operator's desktop against the simulator, and the tray
  shows battery on this desktop environment.
- **Phase 4.** Settings parity: buttons, DPI stages and colors, polling rate, debounce, LOD,
  motion sync, angle snap, ripple, highest performance, sleep time, long range, profiles, battery,
  firmware versions.
- **Phase 5.** Macros: recording, editor, library in the store, binding to buttons.
- **Phase 6.** Receiver tools (light, pairing, factory reset) and config export and import.
- **Phase 7.** Firmware: container parser, identity gate, update state machine, guards, empty-state
  UI, and the three acquisition routes.
- **Phase 8.** Packaging under `dist/`, the udev rule and its first-run check, and user docs.

## 5. Scale and streaming

- **Settings.** The device shadow is 16 KB, read in 10-byte frames and written in 10-byte frames.
  Hyperpace holds exactly one shadow per connected device and never accumulates history in memory.
- **Firmware.** Images are streamed to the device in 32-byte packets read directly from embedded
  bytes, which are file-backed pages on Linux, so a 100 KB to 1 MB image never costs more than the
  packet being sent plus the page being read. The sha256 check streams over the same reader.
- **Store.** defradb.rs reads and scans stream rather than materialize; the macro library and
  profile snapshots are kilobytes.
- **Idle cost.** The webview is destroyed when the window closes to the tray, battery polls every
  5 s, and the tray icon is redrawn only when the displayed bucket changes, because on this
  platform every icon update writes a file and crosses D-Bus.

## 6. Verification

- **Differential oracle.** The vendor JavaScript protocol module runs under bubblewrap with no
  network, filesystem or environment, driven by a mock HID device, and its emitted frames are
  compared byte for byte against `hyperpace-protocol` for every command and setting. Golden
  vectors are committed so the comparison also runs without the sandbox.
- **Device simulator.** Implements the protocol including the vendor's quirks, so app-level tests
  never need hardware.
- **Property tests.** Checksum round-trip, DPI encode and decode per model table, keystroke and
  macro buffer round-trip, export and import round-trip.
- **Gate.** `vertexia gate` (format, clippy with warnings as errors, doc build with warnings as
  errors, tests) plus the frontend build and the no-dash rule, green before any push.
- **End to end.** The app launches, connects to the simulator, applies every setting, exports and
  imports a config, and runs the firmware engine against a simulated bootloader.
- **Honesty.** No claim of hardware verification anywhere. The only real-device evidence will come
  from the operator running Hyperpace read-only themselves.

## 7. Status

| Workstream | State | Note |
|---|---|---|
| Research | done | 137 sources, two verification workflows, JUDGEMENT.md |
| Protocol reference | done | `docs/research/mouse-protocol-v2.md` |
| Firmware spec | done | `docs/research/firmware-update-spec.md` |
| Phase 0 (de-risk spikes) | skipped | operator decision 2026-09-16; risks carried into the phases that need them |
| Phase 1 (protocol codec) | done | frames, commands, model tables, memory map, encodings, buttons, keystrokes, macros, config file; golden vectors and property tests |
| Phase 1 (differential oracle) | done | `make oracle` runs the vendor JavaScript sealed under bubblewrap (network, filesystem and environment all denied, seal verified by probe) and compares 358 operations across 16 categories against the Rust codec; all match byte for byte, and the one deliberate deviation (the vendor's config-import check-byte bug) is asserted explicitly. Not covered: the newer bundle's full setter matrix, the keyboard protocol, and the firmware module |
| Phase 2 (device layer and simulator) | done | owner thread, request matching, push dispatch, hotplug, HID and simulator transports; compile-checked only, never run against hardware |
| Phase 3 (app shell) | done | Tauri 3.0.0-alpha window, tray with battery, hide to tray, low battery notification, autostart, single instance; observed running on the operator's desktop |
| Phase 4 (settings parity) | done | buttons, DPI, polling, debounce, LOD, motion sync, angle snap, ripple, performance, sleep, profiles, long range, DPI indicator |
| Phase 5 (macros) | done | recording, editor and library in the embedded store |
| Phase 6 (receiver tools and config files) | done | receiver light, pairing, factory reset, .bin export and import |
| Phase 7 (firmware) | done as machinery | container parser, identity gate, guards, flash state machine, import path; no genuine package exists, so it has never been exercised and rollback remains impossible |
| Phase 8 (packaging) | done as configuration | dist/deb, rpm, appimage, arch, windows, macos plus the udev rule and README; no package has been built or installed |
| Gate | green | fmt, clippy with warnings as errors, doc build, 303 tests, cargo-deny advisories, bans, licenses and sources, plus the frontend build and type check |

Known gaps still open, recorded so none is dropped silently:

| Gap | Why it matters | State |
|---|---|---|
| Keystroke chord editing on the Buttons screen | A button can be bound to a keystroke, but its content cannot be edited: the contract has no command to read or write the keystroke slot | closed: `get_button_keystroke` reads it (backed by the new `DeviceHandle::read_block`), `set_button`'s existing `keystroke` field writes it, and the Buttons screen now has a modifier-plus-key chord editor and a working media-key binding |
| Pairing progress and profile read-back | Pairing and profile changes are fire and forget; the interface shows the request, not the device's answer, because there is no get command or event for either | closed: `pair_receiver` streams every `GetPairState` poll through a progress channel (phase and seconds left, shown on the Lighting screen), and `read_settings` now queries `GetCurrentConfig` alongside the shadow, shown honestly on the Settings screen (including when the device marks it unsupported) |
| Structured logging | The doctrine requires structured logs with a counter per error path; today a few background failures (tray update, notification, window show) print to stderr | closed: `tracing`/`tracing-subscriber` wired up in `hyperpace-app`, every background `eprintln!` replaced with a leveled `tracing` call (error/warn per the doctrine), only the logging-subsystem's own bootstrap-failure fallback still prints directly |
| Simulator fidelity for pairing, factory reset and receiver light | The simulator acknowledges these with a bare success and models no state, so tests cannot prove the flows behave | closed: the simulator now models a real pairing state machine (progresses through phases, can be configured to fail), factory reset erases its flash and cancels any in-progress session, the receiver light stores and returns what was set, and the existing per-command unsupported override proves the honesty-fence path for a model lacking a feature; covered by new tests through the device layer in both `hyperpace-device` and `hyperpace-app` |
| Device layer on real hardware | Compile-checked and simulator-tested only; never opened against a device | blocked by the operator's rule, by design |
| macOS and Windows paths | Implemented per documentation, never executed on those systems | needs those machines |
| Built packages | Packaging configuration exists under `dist/`; no package has been built or installed | to do |

Deferred, with reasons:

- **Firmware images and rollback capability.** No genuine package exists; acquisition is a
  prerequisite, not an implementation task.
- **Code signing and notarization.** Operator chose unsigned builds for now; consequences are
  macOS warnings, Windows SmartScreen, and Windows toast notifications needing the installer.
- **Keyboard support.** The newer vendor driver carries a second keyboard memory map; out of scope
  for this product.
- **4K receiver dual-MCU chain.** Implemented per spec but untestable without hardware and an
  image.
- **macOS Input Monitoring flow and Windows write-length behavior.** Both need a Mac and a Windows
  machine to confirm; the code handles them per documentation.

## Rejected forks and change log

- Rejected: async I/O layer for HID, because of a reported macOS stall after 30 to 444 exchanges
  with an unmerged fix; it stays a candidate for the hotplug watcher only.
- Rejected: a privileged background daemon owning the device, because it needs the same root
  install as a udev rule with more code and more privilege.
- Rejected: Tauri 2.11 stable, chosen against by the operator in favour of 3.0 alpha for its
  tray with tooltips and no extra system library dependency.
- Rejected: AppImage as the only Linux channel, because the default bundle can start with no
  window on current Mesa, which is what this machine runs.
- Rejected: plain Svelte plus Vite, in favour of the SvelteKit static SPA template.
- Rejected: OS-standard configuration directories, in favour of `$HOME/.hyperpace` on every OS.
- Rejected permanently: flashing any sibling image, because the bootloader performs no model check
  once entered and a foreign image bricks the device.
- 2026-09-16: first issue.
