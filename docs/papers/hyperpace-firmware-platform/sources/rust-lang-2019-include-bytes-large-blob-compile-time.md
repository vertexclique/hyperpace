URL: https://github.com/rust-lang/rust/issues/65818
Additional URLs: https://doc.rust-lang.org/std/macro.include_bytes.html ; https://github.com/rust-lang/rust/issues/103607
Retrieved: 2026-09-15 (issue via `gh issue view`; std docs via WebFetch)
Source type: Rust standard library documentation (tier 1) and rust-lang/rust issue tracker with compiler team comments (tier 2)

## Std docs (WebFetch rendering of https://doc.rust-lang.org/std/macro.include_bytes.html)

> Includes a file as a reference to a byte array.
>
> The file is located relative to the current file (similarly to how modules are found). The provided path is interpreted in a platform-specific way at compile time. So, for instance, an invocation with a Windows path containing backslashes `\` would not compile correctly on Unix.
>
> This macro will yield an expression of type `&'static [u8; N]` which is the contents of the file.

## Issue #65818 metadata

```
Using `include_bytes!` on large binary blobs compiles more slowly than expected | OPEN | 2019-10-25T15:44:03Z | closed null
```

## Verbatim, issue body (alexcrichton, 2019-10-25)

> ```rust
> pub const BYTES: &[u8] = include_bytes!("large-binary-blob");
> ```
>
> and I've been surprised that the compile time of this crate is pretty nontrivial!
>
> For example this crate:
>
> ```rust
> pub const BYTES: &[u8] = &[];
> ```
>
> takes 0.060 seconds to compile on nightly for me. If a multi-megabyte binary (such as `cargo` itself) is included:
>
> ```rust
> pub const BYTES: &[u8] = include_bytes!("/path/to/.cargo/bin/cargo");
> ```
>
> this crate takes 0.729 seconds to compile!

> 2. Later in compilation while emitting metadata when we're [processing constants](...)

(item 1 of the list is struck through in the current issue body, meaning that per-byte expansion cost was addressed; item 2 was truncated in the capture)

## Verbatim, comments

alexcrichton (2019-10-28):
> After seeing the const-ness of needing this I tried out a few different formulations as well:
>
> ```rust
> pub static FOO: &[u8] = include_bytes!("...");
> ```
> ...
> unfortunately though while they were *slightly* faster they were still quite slow (on the order of hundreds of milliseconds for a 10MB file or so)

nagisa (2019-10-26):
> Most ideally this would result in rustc telling linker to link that file into whatever symbol. Possible but super non-trivial to achieve.

kennykerr (2021-05-28):
> I just came across this with the [windows](https://github.com/microsoft/windows-rs/) crate where I embed the default metadata describing the Windows API amounting to around 20MB. Are there any plans to fix this or should I write my own implementation?

petrochenkov (2021-05-28):
> I tried to experiment with this last year, there are very real benefits from not escaping bytes from `include_bytes` and not turning them into a `Symbol`, even if nothing else is done (maybe 2x-10x speed/memory by order of magnitude, I don't remember exactly).

## Issue #103607 metadata

```
ICE when include_bytes-ing ~>1GB of data in lib.rs | CLOSED | 2022-10-27T00:11:54Z | 2023-08-30T04:29:14Z
```
