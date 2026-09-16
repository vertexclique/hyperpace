# Judgement: Hyperpace firmware and platform research

Date: 2026-09-16. Gate: phase-gate. This file is the handoff to planning. Forward-only: changes
go at the bottom, history is not rewritten.

## Thesis

Build Hyperpace on the production mouse driver's protocol (cid 102, documented byte-exact in
`docs/research/mouse-protocol-v2.md`) with a corrected, serialized transaction layer, reaching the
device through hidapi with a separate hotplug watcher, packaged as Tauri 2.11 with Svelte 5 and
native Linux packages first. Implement the firmware engine to the verified spec
(`docs/research/firmware-update-spec.md`), and ship it with an empty, honest archive: no genuine
firmware for this mouse exists on any channel reached, so install and rollback have nothing to
install until a package is acquired.

## Evidence base

| Stream | Output |
|---|---|
| vendor, chips, tauri, hid | 137 sources, 137 summaries, 2 search logs |
| protocol cross-check (44 agents) | 401 facts: 304 confirmed, 96 corrected, 1 refuted; `docs/research/mouse-protocol-v2.md` |
| firmware verification (27 agents) | spec 24 confirmed / 14 corrected; 100 gap facts; 14 claims, 13 corrected, 1 unverifiable; 37 firmware candidates, 0 for this mouse; `FIRMWARE-VERDICT.md`, `docs/research/firmware-update-spec.md` |
| local primary data | HID report descriptors from sysfs; vendor page deployments compared by sha256 |

Both workflows ended with their completeness critics still reporting new gaps (10 and 9 in the
final rounds). Coverage is good but not exhaustive, and every negative below means "not found on
the channels reached", never "does not exist".

## Ranked verdict

| Area | Choice | Correctness basis | Fit | Verdict |
|---|---|---|---|---|
| Settings protocol | OLD driver, cid 102 tables, v2 reference | 304 of 401 facts confirmed by two independent verifiers, rest corrected; /mouse assets hash-identical to the analyzed bundle | Exactly this product | Build on |
| Transaction layer | Hyperpace's own, not a port of the vendor's | Vendor accepts a 3-byte echo, treats error replies as success, does not serialize requests, lets a push consume a retry and duplicate a write | Vendor behavior is a correctness bug, not a requirement | Build on, corrected |
| Model selection | Runtime cid and mid lookup over both tables | Two vendor configs disagree (102 vs 62; 5 vs 6 buttons; 40000 vs 32000 DPI; different 3950 ranges) and only the device can settle it | Operator now permits read-only access when they run it | Build on |
| HID access | hidapi-rs (linux-native-basic-udev, windows-native, non-exclusive macOS), one owner thread per device | Library contract and source read; Windows write length 17 derived from this unit's descriptors | Matches the request/response channel | Build on |
| Hotplug | Separate watcher (async-hid watch or nusb watch_devices), start watcher then enumerate | hidapi has no hotplug API and none is released upstream | Needed for receiver, cable, sleep | Build on |
| macOS I/O | hidapi, not async-hid 0.5.3 | Reported stall after 30 to 444 exchanges, unmerged fix | Battery polling matches the failing pattern | Partial: revisit when fixed |
| Packaging | Tauri 2.11 + Svelte 5; deb and rpm first on Linux, AppImage tested and pruned | AppImage bundles display libraries that abort WebKit on current Mesa, which is what this machine runs | Single binary with embedded assets holds on all three | Build on, AppImage at risk |
| Tray and battery | Icon-rendered percentage plus menu item; always attach a menu | Linux tray has no tooltip or click events and hides without a menu | Background battery display is a core requirement | Build on |
| Notifications | Send from Rust; expect GNOME breakage | Plugin drops the handle, GNOME 46+ closes on sender disappearance; Windows toasts need an installed app | Low-battery warning is required | Build on, with caveats |
| Firmware bundling | `include_bytes!` plus a SHA-256 manifest compiled into the signed binary | Only route keeping one binary on every OS; Tauri verifies nothing itself | Required by the operator | Build on when packages exist |
| Firmware install | Compx ComUsbUpgradeFile engine per the verified spec | Three implementations agree; packets confirmed against two real vendor USB logs | Required | Build, cannot be exercised |
| Firmware rollback | Not achievable today | Needs two or more vendor-built packages per target; archive holds zero, and no read-back exists to snapshot the installed version | Required by the operator | Blocked on acquisition |
| Sibling firmware images | Never flash | 35 archived images carry foreign PIDs and model ids; the bootloader performs no model check once entered | Reference only | Reject |

## The honesty fence

1. **No firmware exists to bundle.** 37 candidates across vendor sites, CDNs, archives, code hosts
   and Chinese channels; all five archive slots for this hardware are empty. Install and rollback
   ship with nothing to install, and the UI must say "no firmware available for this device yet",
   never a false green.
2. **Rollback cannot be manufactured.** No implementation synthesizes the prepare packet; all copy
   it from a vendor header, and the check value for this mouse's MCU family matches no tested CRC
   scheme. A dump, a raw payload or a rebuilt image is unflashable. Each rollback target must
   arrive as a complete vendor package.
3. **Nothing was observed on the wire.** Every protocol statement is code-derived. Flashing is
   unproven on hardware and must never be described as verified.
4. **Three HYPACE-specific unknowns remain even with the engine written**: the boot-mode PID, the
   report ids and output-versus-feature flag, and the prepare check value. All three are readable
   only from a genuine image.
5. **Coverage is incomplete** in both workflows, as stated above.

## Needs an experiment

- Does a Tauri AppImage launch on this machine (Mesa, Wayland and X11, each GPU), and does pruning
  the bundled display libraries fix it?
- Idle CPU and memory with the webview hidden versus destroyed.
- Windows: the actual report-8 write length and collection selection on real Windows.
- macOS: whether the receiver's vendor interface triggers Input Monitoring, as its Generic Desktop
  Mouse collection suggests, while the wired interface does not.
- Which cid and mid this mouse reports, once the operator runs Hyperpace read-only.

## Open questions for planning

1. Acquisition: ask the vendor for per-version packages (the only route that can yield rollback),
   poll the two config files and the directory oracle, accept operator-supplied packages behind a
   strict identity gate. Which does the operator want, and who contacts the vendor?
2. Degraded-state behavior with an empty archive: what the firmware screen shows and offers.
3. Linux distribution channel: deb and rpm, AppImage, or an AUR package for this machine.
4. Tauri 2.11 versus the 3.0 alpha (ksni tray with tooltips, no libayatana) for a tray-centric app.
5. Whether Hyperpace should watch for firmware appearing and tell the operator.
6. macOS and Windows signing identities, which gate notifications and Gatekeeper.

## Changes

- 2026-09-16: first issue.
