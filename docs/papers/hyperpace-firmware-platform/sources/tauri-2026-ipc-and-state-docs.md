URL: https://v2.tauri.app/develop/calling-rust/
Additional URLs: https://v2.tauri.app/develop/calling-frontend/ ; https://v2.tauri.app/develop/state-management/ ; https://github.com/tauri-apps/tauri-docs/tree/v2/src/content/docs/develop (commit a6b59b78) ; https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/CHANGELOG.md
Retrieved: 2026-09-15
Source type: official documentation and changelog (primary)

## Verbatim, calling-rust: async commands

> Asynchronous commands are preferred in Tauri to perform heavy work in a manner that doesn't result in UI freezes or slowdowns.
>
> :::note
>
> Async commands are executed on a separate async task using [`async_runtime::spawn`].
> Commands without the _async_ keyword are executed on the main thread unless defined with _#[tauri::command(async)]_.

> You need to be careful when creating asynchronous functions using Tauri.
> Currently, you cannot simply include borrowed arguments in the signature of an asynchronous function.
> Some common examples of types like this are `&str` and `State<'_, Data>`.
> This limitation is tracked here: https://github.com/tauri-apps/tauri/issues/2533 and workarounds are shown below.

> **Option 2**: Wrap the return type in a [`Result`]. This one is a bit harder to implement, but works for all types.

## Verbatim, calling-rust: channels

> The Tauri channel is the recommended mechanism for streaming data such as streamed HTTP responses to the frontend.
> The following example reads a file and notifies the frontend of the progress in chunks of 4096 bytes:

```rust
#[tauri::command]
async fn load_image(path: std::path::PathBuf, reader: tauri::ipc::Channel<&[u8]>) {
```

## Verbatim, calling-rust: events

> The event system is a simpler communication mechanism between your frontend and the Rust.
> Unlike commands, events are not type safe, are always async, cannot return values and only supports JSON payloads.

## Verbatim, calling-frontend

> The event system is not designed for low latency or high throughput situations.

> event payloads are always JSON strings making them not suitable for bigger messages

> The event system is designed to be a simple two way communication that is globally available in your application.
> Under the hood it directly evaluates JavaScript code so it might not be suitable to sending a large amount of data.
>
> Channels are designed to be fast and deliver ordered data. They are used internally for streaming operations
> such as download progress, child process output and WebSocket messages.

```rust
use tauri::{AppHandle, ipc::Channel};
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "event", content = "data")]
enum DownloadEvent<'a> {
  Started {
    url: &'a str,
    download_id: usize,
    content_length: usize,
```

## Verbatim, state-management

```rust
app.manage(Mutex::new(AppState::default()));
```

> ### When to use an async mutex
>
> To quote the [Tokio documentation](...), it's often fine to use the standard library's [`Mutex`] instead of an async mutex such as the one Tokio provides:
>
> > Contrary to popular belief, it is ok and often preferred to use the ordinary Mutex from the standard library in asynchronous code ... The primary use case for the async mutex is to provide shared mutable access to IO resources such as a database connection.
>
> It's a good idea to read the linked documentation fully to understand the trade-offs between the two. One reason you _would_ need an async mutex is if you need to hold the `MutexGuard` across await points.

> ### Do you need `Arc`?
>
> ... However, you don't need to use [`Arc`] for things stored in [`State`] because Tauri will do this for you.
>
> In case `State`'s lifetime requirements prevent you from moving your state into a new thread you can instead move an `AppHandle` into the thread and then retrieve your state as shown below ... `AppHandle`s are deliberately cheap to clone for use-cases like this.

## Verbatim, tauri CHANGELOG 2.11.3 (Bug Fixes)

> [`66f873d62`] ([#15491] by [@bclarke123]) Adjust mutex locking in `send_channel_data_handler`, `handle_android_plugin_response`, `send_channel_data` to avoid deadlocks

## Verbatim, dev `.changes` titles related to IPC listeners (unreleased)

```
event-listener-cleanup-on-destroy.md
fix-js-listeners-leak-on-webview-close.md
fix-unlisten-guard-missing-entry.md
```
(file names only; contents not read)
