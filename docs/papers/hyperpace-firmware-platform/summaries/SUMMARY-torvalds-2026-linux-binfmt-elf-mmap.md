# SUMMARY: torvalds-2026-linux-binfmt-elf-mmap

## Claim
The Linux ELF loader maps each loadable segment of an executable with `vm_mmap` against the executable file (a private mapping), rather than copying the file into memory.

## Method
Read `fs/binfmt_elf.c` (`elf_map`, `elf_load`) on the kernel master branch.

## Result
Segments, including the read-only data that holds `include_bytes!` content, are file-backed mappings.

## Evidence tier
1.

## Performance
File-backed pages are populated on first access and can be evicted under memory pressure. This is general mmap behaviour, not restated in the excerpt.

## Correctness
- **Proven:** mapping via `vm_mmap`.
- **Assumed:** demand-paging semantics.
- Inside an AppImage the file is served by the squashfs FUSE mount, so first access also costs decompression.
- Windows and macOS loaders were not examined.

## Relevance to hyperpace
On Linux, tens of MB of embedded firmware cost disk and download size, not idle RAM. Reading a firmware image to flash it touches only that image's pages.
