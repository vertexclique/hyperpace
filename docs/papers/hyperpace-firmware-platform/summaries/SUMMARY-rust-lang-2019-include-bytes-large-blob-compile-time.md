# SUMMARY: rust-lang-2019-include-bytes-large-blob-compile-time

## Claim
`include_bytes!` yields `&'static [u8; N]` holding the file contents at compile time. Including multi-megabyte blobs compiles more slowly than expected:
- a crate embedding the cargo binary took 0.729 s, against 0.060 s empty (2019);
- around 10 MB took "hundreds of milliseconds".

The issue is still open. Around 1 GB caused an ICE (closed 2023).

## Method
The std docs, and a compiler issue with measurements by a Rust team member plus later comments (windows-rs embeds about 20 MB).

## Result
The per-byte token expansion cost has been addressed (struck item). Metadata and const handling remain the reported cost.

## Evidence tier
1 (std docs), 2 (compiler team issue).

## Performance
2019 numbers only; the current rustc was not re-measured.

## Correctness
By these numbers, compile-time cost at tens of MB is modest, but this is unverified on rustc 1.98.

## Relevance to hyperpace
Embedding tens of MB of firmware with `include_bytes!` is workable. Put the blobs in a small dedicated crate so that edits to app code do not re-process them, and measure clean and incremental build times.
