URL: https://raw.githubusercontent.com/tauri-apps/benchmark_results/gh-pages/tauri-recent-linux.json
Additional URLs: https://raw.githubusercontent.com/tauri-apps/benchmark_results/gh-pages/tauri-data-linux.json ; https://github.com/tauri-apps/tauri/blob/dev/bench/src/run_benchmark.rs ; https://github.com/tauri-apps/tauri/blob/dev/bench/src/utils.rs ; https://github.com/tauri-apps/tauri/blob/dev/bench/README.md
Retrieved: 2026-09-15 (benchmark_results repo default branch `gh-pages`, pushed_at 2026-09-15T06:57:46Z)
Source type: official CI benchmark data plus the benchmark harness source (primary, but self-described internal)

## Verbatim, newest entry of tauri-recent-linux.json (20 entries in file; tauri-data-linux.json has 1559)

```json
{
 "binary_size": {
  "tauri_3mb_transfer": 2956616,
  "tauri_cpu_intensive": 2952544,
  "tauri_hello_world": 2951064,
  "wry_rlib": 1322862
 },
 "cargo_deps": {
  "Linux": 823,
  "Windows": 752,
  "macOS": 741
 },
 "created_at": "2026-09-15T06:51:39.604992187Z",
 "exec_time": {
  "tauri_hello_world": {
   "max": 0.76572913332,
   "mean": 0.7574049101200002,
   "min": 0.75088663432,
   "stddev": 0.00463611988597972,
   "system": 0.0592208,
   "user": 0.09992642
  }
 },
 "max_memory": {
  "tauri_3mb_transfer": 395509760,
  "tauri_cpu_intensive": 527298560,
  "tauri_hello_world": 396345344
 },
 "sha1": "bca4ca58da02f182ef00ef1165e40e400a3cd8dd",
 "syscall_count": {
  "tauri_hello_world": 23839
 },
 "thread_count": {
  "tauri_3mb_transfer": 77,
  "tauri_cpu_intensive": 83,
  "tauri_hello_world": 77
 }
}
```
(exec_time and syscall_count for the two other benchmarks omitted here for length; they are in the file.)

## Trend (computed by this stream from tauri-data-linux.json: first entry of each calendar year, or the January entry when present; not a statistic published by Tauri)

```
2021 2021-05-21 hello_bin 2306480 hello_maxmem 191889408 threads 47
2022 2022-04-22 hello_bin 2766424 hello_maxmem 244318208 threads 42
2023 2023-01-29 hello_bin 1954056 hello_maxmem 307232768 threads 42
2024 2024-01-31 hello_bin 3579616 hello_maxmem 443547648 threads 69
2025 2025-01-01 hello_bin 2649176 hello_maxmem 281018368 threads 70
2026 2026-06-09 hello_bin 2943328 hello_maxmem 395300863 threads 77
```

## Verbatim, harness: how max_memory is measured (bench/src/run_benchmark.rs)

```rust
fn run_max_mem_benchmark(target: &str) -> Result<HashMap<String, u64>> {
  let mut results = HashMap::<String, u64>::new();

  for (name, example_exe) in get_all_benchmarks(target) {
    let benchmark_file = utils::target_dir().join(format!("mprof{name}_.dat"));
...
    let proc = Command::new("mprof")
      .args(["run", "-C", "-o", benchmark_file_str, exe_path_str])
```

```rust
  if cfg!(target_os = "linux") {
    println!("Running Linux-specific benchmarks...");
    run_strace_benchmarks(&mut new_data, target)?;
    new_data.max_memory = run_max_mem_benchmark(target)?;
  }
```

## Verbatim, bench/src/utils.rs parse_max_mem (takes the highest sample)

```rust
  for line in output.lines().map_while(Result::ok) {
    let split: Vec<&str> = line.split(' ').collect();
    if split.len() == 3 {
      if let Ok(mb) = split[1].parse::<f64>() {
        let current_bytes = (mb * 1024.0 * 1024.0) as u64;
        highest = highest.max(current_bytes);
      }
    }
  }
```

## Verbatim, bench/README.md

> This rust module run on CI, provides internal metrics results of Tauri. To learn more see [benchmark_results](https://github.com/tauri-apps/benchmark_results) repository.
>
> **\*_Internal use only_**
