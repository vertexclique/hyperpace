URL: https://github.com/carlossless/hidra ; https://crates.io/crates/hidra ; https://docs.rs/hidra/latest/hidra/
Retrieved: 2026-09-15
Source type: README (GitHub API), crates.io registry API (raw JSON), docs.rs page (WebFetch extraction).
Retrieval method: `gh api repos/carlossless/hidra/readme`; `curl https://crates.io/api/v1/crates/hidra`; WebFetch docs.rs.
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

crates.io API (raw):
```
max 0.0.4 created 2026-07-09T18:36:25.413875Z updated 2026-09-03T19:57:39.637154Z dl 733 repo https://github.com/carlossless/hidra
  0.0.4 2026-09-03T19:57:39.637154Z
  0.0.3 2026-08-27T05:39:35.016968Z
  0.0.2 2026-07-10T15:27:47.782845Z
  0.0.1 2026-07-09T18:36:25.413875Z
```
GitHub API: `hidra pushed_at=2026-09-03T19:57:26Z stars=1 open_issues=0`

README:
> A pure-Rust HID library: one async API across Linux, Windows, macOS and the
> browser, with blocking `.wait()` on native targets, plus standalone HID
> report-descriptor primitives.

| Platform | `Native` | Notes |
|---|---|---|
| Linux | `hidraw` device nodes, sysfs enumeration | no libudev dependency |
| Windows | `hid.dll` + SetupAPI (via `windows-sys` declarations) | |
| macOS | IOHIDManager (direct framework FFI) | |
| Browsers | WebHID via `web-sys` | same types, await-only |

> `Nusb` adds raw USB transfers via nusb on the three native targets

docs.rs extraction (tool summary, not verbatim): "No hotplug/watch support is mentioned in the documentation".
`gh search code --repo carlossless/hidra "hotplug OR watch"` returned no results (code search may not index very new repositories, so absence is weak evidence).

Also surfaced by the same search and not evaluated further: `keyroost-hid` 0.9.0 (created 2026-06-13, 718 downloads), described in search results as using the hidapi crate on macOS and Windows.
