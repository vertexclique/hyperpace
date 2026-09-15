URL: https://github.com/tauri-apps/tauri/issues/5889
Retrieved: 2026-09-15 (via `gh issue view 5889 --repo tauri-apps/tauri`)
Source type: GitHub issue, community measurement with maintainer replies (state CLOSED, opened 2022-12-21)

Title: "Memory benchmark might be incorrect: Tauri might consume more RAM than Electron"

## Verbatim, issue body (jviotti)

> The current memory benchmark (https://tauri.app/v1/references/benchmarks/) finds Electron to have a ~500 MB memory usage on Linux, being 2x from the one from Tauri.
>
> The problem might be related to failing to account for shared memory, given that Chromium uses the same executable for multiple of its processes, with a significant part of them being read-only and thus actually shared. If we take these into account, the results are very different:

| postman.com  | macOS 12.6.1  | Ubuntu 22.04.1  | Windows 10 |
|---|:--:|:--:|:--:|
| **Tauri**                   | 421 MB | 581 MB | 399 MB |
| **Safari**                 |  471 MB  |  :x:  |  :x:  |
| **Electron**             |  337 MB  | 240 MB | 318 MB |
| **Chrome**                 | 381 MB | 370 MB |  <not done>  |

| vscode.dev  | macOS 12.6.1  | Ubuntu 22.04.1  | Windows 10 |
|---|:--:|:--:|:--:|
| **Tauri**                   | 429 MB | 572 MB | 370 MB |
| **Electron**             | 332 MB | 222 MB | 312 MB |

## Verbatim, methodology reply (jviotti, 2023-01-02)

> - We closed unnecessary apps running on the system. Before running tests observe memory usage on the system and confirm it's mostly stable
> - On Ubuntu, we measured using "System Monitor", `free` and `sme

(truncated in capture at 600 characters)

## Verbatim, maintainer-side reply (Beanow, 2022-12-28)

> So first the disclaimer: the benchmarks on https://tauri.app/v1/references/benchmarks/ need to be taken with a good handful of salt. It's by no means a scientific report and honestly only should be considered a smoke test / regression test. Where the main usefulness in running these benchmarks is to compare one Tauri version against itself, given similar testing conditions.
