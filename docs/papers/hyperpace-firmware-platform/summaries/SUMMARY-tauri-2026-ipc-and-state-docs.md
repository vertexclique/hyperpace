# SUMMARY: tauri-2026-ipc-and-state-docs

## Claim
- **Commands:** async commands run on `async_runtime::spawn` (tokio). Non-async commands run on the main thread unless marked `#[tauri::command(async)]`. Async commands cannot take borrowed args like `&str` or `State<'_, T>` unless they return `Result`.
- **Channels:** `tauri::ipc::Channel<T>` is "designed to be fast and deliver ordered data" and is recommended for streaming.
- **Events:** JSON-only, not type safe, and "not designed for low latency or high throughput".
- **State:** managed state is wrapped in `Arc` by Tauri. A std `Mutex` is fine unless held across `.await`.

## Method
The official develop docs and the changelog.

## Result
- 2.11.3 fixed deadlock-prone mutex locking in channel send paths.
- Pending dev changes fix JS listener leaks on webview close.

## Evidence tier
1.

## Performance
The docs give no throughput numbers.

## Correctness
Guidance level. The listener-leak fixes are unreleased (only the file names were read).

## Relevance to hyperpace
- **Device state** (battery, DPI, connection) at a low rate: a `subscribe(channel)` command per window, re-subscribed when a destroyed window is recreated.
- **Events** are acceptable for rare global notices.
- **HID I/O:** long-running I/O should live in a dedicated Rust task that owns the device. Commands send requests to it over a channel. This also fits the doctrine's preference for message passing over shared locks.
