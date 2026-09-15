# SUMMARY: dclong-2021-include-bytes-memory-forum

## Claim
`include_bytes!` data lands in the executable's data section (typically `.rodata`). Because it is read-only, its physical pages can be shared by every process that maps the same file.

## Method
Forum answers.

## Result
The thread does not say explicitly that pages are loaded lazily.

## Evidence tier
3.

## Performance
Not measured.

## Correctness
Consistent with the ELF loader mapping segments from the file (see torvalds-2026-linux-binfmt-elf-mmap). Lazy population is general OS behaviour, not proven here.

## Relevance to hyperpace
Embedded firmware should not raise resident memory until it is read. Verify with an RSS check of the built binary.
