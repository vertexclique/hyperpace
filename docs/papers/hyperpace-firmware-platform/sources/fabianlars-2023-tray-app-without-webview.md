URL: https://github.com/tauri-apps/tauri/discussions/6308
Retrieved: 2026-09-15 (via GitHub GraphQL API)
Source type: GitHub discussion with an accepted answer by a Tauri core maintainer (FabianLars), created 2023-02-18

Title: "Tray app without WebView processes when hidden"

## Verbatim, question (TheFranconianCoder)

> Hi, when I build an app that mostly sits in the tray, only hiding the WebView still costs this ressources. Is it possible to start the WebView processes only on demand, when any action triggers it.

## Verbatim, accepted answer (FabianLars, 2023-02-18T19:01:14Z)

> You need to close the webview _window_, and create the window on demand (like on tray menu events) - both of course in Rust because without a webview there is no JS process/runtime.
>
> you can keep the app running without a window like this: <https://github.com/tauri-apps/tauri/blob/b02fc90f450ff9e9d8a35ee55dc1beced4957869/examples/api/src-tauri/src/main.rs#L240-L244>

## Verbatim, related maintainer comment on process layout (FabianLars, discussion #11553, 2024-11-05, https://github.com/tauri-apps/tauri/discussions/11553)

> `tauri://localhost` is the main browser process (similar to a tab process in your browser).
> the second one should be the rust process.
> the other 2 are obviously the webview's/browser's sub-processes for gpu and network.
