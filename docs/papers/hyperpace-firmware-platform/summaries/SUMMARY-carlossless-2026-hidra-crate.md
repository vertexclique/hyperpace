# SUMMARY: hidra 0.0.4

## Claim
hidra is a new pure-Rust HID library with one async API and blocking `.wait()`:
- Linux: hidraw with sysfs enumeration, no libudev.
- Windows: hid.dll and SetupAPI.
- macOS: IOHIDManager.
- Browser: WebHID.
- An optional Nusb backend for raw USB transfers.

## Method
README, crates.io JSON and docs.rs extraction. A code search for hotplug returned nothing.

## Result
- Created 2026-07-09; 0.0.4 released 2026-09-03; 733 downloads; 1 star; no issues.
- No hotplug or watch API found.

## Evidence tier
1 for what it claims about itself; maturity is effectively unproven.

## Performance
Not evaluated.

## Correctness
- Too new to judge.
- Absence of hotplug is weakly established: GitHub code search may not index a new repository.

## Relevance to hyperpace
Not recommended for a product now. Worth rechecking later: a unified Native/Nusb backend type could suit the HID channel plus a USB bootloader.
