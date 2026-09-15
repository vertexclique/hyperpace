# SUMMARY: tauri-plugin-hid

## Claim
A third-party Tauri plugin exposes HID enumeration, multi-device open and input/output report I/O through hidapi-rs on macOS, Windows and Linux, and Android UsbManager on Android. It has no feature report support and is "only tested on macOS, Windows and Android".

## Method
README and crates.io metadata. Source not read.

## Result
- 0.2.3, last updated 2026-01-07, 6,227 downloads.
- It demonstrates hidapi-rs inside Tauri 2, but with no Linux testing and no feature reports.

## Evidence tier
1 for its self-description, 3 as evidence of fitness.

## Performance
Not evaluated.

## Correctness
- Unknown how it handles the macOS long-lived-init-thread requirement (hidapi-rs issue 127). Not read.

## Relevance to hyperpace
- Not suitable as-is: HYPACE has feature report 6, Linux is a target, and the protocol logic belongs in Rust, not in the frontend.
- Its source could be read for Tauri integration ideas.
- hyperpace should own its HID thread directly.
