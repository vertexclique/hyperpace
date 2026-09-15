URL: https://github.com/tauri-apps/tauri/issues/14088
Retrieved: 2026-09-15 (via `gh issue view 14088 --repo tauri-apps/tauri`)
Source type: GitHub issue (community report, maintainer triage and fix pointer); state CLOSED; opened 2025-08-26

Title: "[bug] Application crashes after all windows are hidden"

## Verbatim, body (mg-chao)

> All windows were hidden in the background using the window.hide() method, and the application crashed approximately 50 minutes later.
>
> The application tray remains, but the window cannot be restored. Upon checking Task Manager, all pages of the webview have been closed. The application backend continues to run, but its memory usage is significantly lower than normal.

> Reports have been received on both Windows 10 and Windows 11 systems. ... It seems that the application does not crash when the main window is kept in the foreground. The issue often occurs after the application has been left idle for a long time, and reproducing the issue is inconsistent.

## Verbatim, panic observed (mg-chao, 2025-08-30)

> thread 'main' panicked at C:\Users\Magic\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tao-0.34.2\src\platform_impl\windows\event_loop.rs:2345:11:
> assertion failed: flush_paint_messages(None, &subclass_input.event_loop_runner)

## Verbatim, mitigation report (acx0, 2025-12-15)

> We haven't had any users hit that assertion after we updated `tray-icon` from 0.21.1 to [0.21.2](https://github.com/tauri-apps/tray-icon/releases/tag/tray-icon-v0.21.2)

## Verbatim, maintainer close comment (Legend-Master, 2026-06-02)

> This should be fixed by https://github.com/tauri-apps/tray-icon/pull/284, upgrade `tray-icon` to `>=0.21.2` to apply the fix
>
> Feel free to re-open if it still happens to you
