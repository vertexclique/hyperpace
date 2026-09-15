# SUMMARY: tauri-2026-benchmark-results-linux

## Claim
Tauri's CI benchmark on Linux at commit bca4ca58 (2026-09-15) reports, for hello world:
- binary: 2,951,064 bytes (about 2.8 MiB);
- peak memory: 396,345,344 bytes (about 378 MiB);
- 77 threads;
- 0.757 s mean execution time.

## Method
- `mprof run -C` (includes child processes) on the bench apps; the harness keeps the highest sample.
- Memory is measured on Linux only.
- Data lives on the `gh-pages` branch of tauri-apps/benchmark_results.

## Result
Yearly samples computed by this stream:
- Peak memory grew from 191,889,408 bytes (2021-05) to 395,300,863 bytes (2026-06).
- Binary size stayed between 1.95 MB and 3.58 MB.

## Evidence tier
1 for the data and the harness. However, the harness README says "Internal use only", and a maintainer called the benchmarks a "smoke test / regression test" (see jviotti-2022).

## Performance
- **Binary:** about 2.8 MiB for hello world.
- **Memory:** a peak during a short scripted run, summed over processes; not idle memory.

## Correctness
- It is not idle memory.
- RSS summed across WebKit processes over-counts shared pages.
- The build profile and machine are not documented in what was read.
- There is no Windows or macOS memory data.
- Do not quote it as "Tauri uses X MB idle".

## Relevance to hyperpace
It gives a binary-size baseline before firmware blobs. The idle cost of a tray app with a hidden or destroyed webview is not measured anywhere found. That needs a hyperpace-specific experiment: PSS via smem, per process, hidden vs destroyed window, on each OS.
