# SUMMARY: apple-tn2206-code-signing-in-depth

## Claim
- Since OS X 10.9, code signatures seal all files in a bundle, with no way to exclude parts.
- Bundles should be treated as read-only after signing.
- Non-Mach-O data belongs in `Contents/Resources`.
- Appending data to a signed Mach-O executable is prohibited and fails verification.

## Method
An Apple technical note (archived).

## Result
Firmware placed in `Contents/Resources`, or compiled into the executable, is covered by the app's signature.

## Evidence tier
1.

## Performance
Not applicable.

## Correctness
The document is archived but still consistent with current practice. When and how often macOS re-validates sealed resources at runtime (first launch vs every launch) is not covered by the excerpt and was not verified.

## Relevance to hyperpace
On a signed and notarized macOS build, both `include_bytes!` and resources are tamper-evident at the OS level. Never write downloaded firmware into the `.app`; keep downloads in the app data directory, with their own verification.
