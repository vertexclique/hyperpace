URL: https://github.com/torvalds/linux/blob/master/fs/binfmt_elf.c
Retrieved: 2026-09-15 (raw file via curl from raw.githubusercontent.com; latest commit touching the file per GitHub API: ed3b875bea55a3ec4837113356df2ead11115af9, 2026-08-21T01:17:08Z)
Source type: Linux kernel source code (primary)

## Verbatim, `elf_map` (fs/binfmt_elf.c lines ~363-400)

```c
/*
 * Map "eppnt->p_filesz" bytes from "filep" offset "eppnt->p_offset"
 * into memory at "addr". (Note that p_filesz is rounded up to the
 * next page, so any extra bytes from the file must be wiped.)
 */
static unsigned long elf_map(struct file *filep, unsigned long addr,
		const struct elf_phdr *eppnt, int prot, int type,
		unsigned long total_size)
{
	unsigned long map_addr;
	unsigned long size = eppnt->p_filesz + ELF_PAGEOFFSET(eppnt->p_vaddr);
	unsigned long off = eppnt->p_offset - ELF_PAGEOFFSET(eppnt->p_vaddr);
	addr = ELF_PAGESTART(addr);
	size = ELF_PAGEALIGN(size);
...
	if (total_size) {
		total_size = ELF_PAGEALIGN(total_size);
		map_addr = vm_mmap(filep, addr, total_size, prot, type, off);
		if (!BAD_ADDR(map_addr))
			vm_munmap(map_addr+size, total_size-size);
	} else
		map_addr = vm_mmap(filep, addr, size, prot, type, off);
```

## Verbatim, mapping type lines

```
674:			int elf_type = MAP_PRIVATE;
1055:		elf_flags = MAP_PRIVATE;
```

## Scope note

The source shows that loadable ELF segments are established with `vm_mmap` against the executable file (a private file-backed mapping). That file-backed mappings are populated lazily by page faults is general Linux mmap behaviour and is not restated in this excerpt. Windows PE and macOS Mach-O loaders were not examined by this stream.
