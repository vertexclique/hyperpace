# SUMMARY: jviotti-2022-tauri-memory-benchmark-shared-memory

## Claim
When shared memory is accounted for, a Tauri app loading heavy web apps can use more RAM than Electron. On Ubuntu 22.04.1:

| Site | Tauri | Electron |
|---|---|---|
| postman.com | 581 MB | 240 MB |
| vscode.dev | 572 MB | 222 MB |

## Method
The reporter measured with System Monitor, `free` and `smem` after letting the system settle. The workloads were heavy remote sites, not local SPAs.

## Result
Tables cover macOS 12.6.1, Ubuntu 22.04.1 and Windows 10. A maintainer-side reply called Tauri's own benchmarks a smoke test, useful only for comparing Tauri against itself.

## Evidence tier
3 (community measurement), with a tier 2 maintainer disclaimer.

## Performance
The Linux Tauri numbers were the worst of the three OSes in both tables.

## Correctness
- 2022 data, on Ubuntu 22.04's WebKitGTK of that time.
- Heavy remote workloads.
- Methodology only partly described, and not reproduced by this stream.

It shows that "small memory" is not a given on Linux. It is not a number for hyperpace.

## Relevance to hyperpace
Do not promise low RAM on Linux based on framework marketing. For a background tray app, the decisive lever is whether a WebKit web process exists while the window is hidden. Measure with PSS.
