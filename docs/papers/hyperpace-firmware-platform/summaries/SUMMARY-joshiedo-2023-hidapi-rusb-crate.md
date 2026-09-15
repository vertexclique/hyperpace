# SUMMARY: hidapi-rusb 1.3.3

## Claim
hidapi-rusb is a separate crate whose last release (1.3.3) and last repository push were on 2023-11-24. It has 3.08 M downloads, which suggests it is a transitive dependency of something popular.

## Method
crates.io registry JSON and GitHub metadata only. The README and source were not read.

## Result
- Inactive for almost three years as of 2026-09-15; no open issues; not archived.
- Its description as a rusb/libusb-based hidapi-rs fork comes from its name and repository and is unverified here.

## Evidence tier
1 (registry metadata). The characterization is unverified.

## Performance
Not evaluated.

## Correctness
- Proven: dates and counts.
- Everything about backends is an assumption.

## Relevance to hyperpace
- Not recommended: unmaintained, and a libusb-based HID path loses usage-page information on Linux and needs libusb at runtime.
- No reason found to prefer it over hidapi 2.6.7.
