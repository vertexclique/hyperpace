URL: https://users.rust-lang.org/t/memory-consume-of-include-bytes/59237
Retrieved: 2026-09-15 (WebFetch; the tool returned quoted posts plus its own analysis, only the quoted posts are reproduced here)
Source type: Rust users forum thread (community, tier 3)

## Question (dclong, 2021-05-02)

> If I use `include_bytes!` to bundle a data file into a Rust executable, will the data consume physical memory? Or does it use some kind of virtual memory?

## Answers

erelde (2021-05-02):
> It'll probably be included in the data segment of the compiled executable, use `objdump` on your compiled artefact to check. (edit: yes it will use memory, 'virtual' memory is an OS abstraction)

cuviper (2021-05-02):
> If it ends up in the read-only `.rodata` section, there's a benefit that any number of running processes will share the same physical memory pages for that data, just mapped to their own virtual memory addresses.

## Note

The thread does not state explicitly whether the pages are loaded at startup or on first access. See the Linux ELF loader source file (torvalds-2026-linux-binfmt-elf-mmap.md) for the file-backed mapping.
