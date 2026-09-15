# SUMMARY: tauri-2026-app-lifecycle-api-source

## Claim
Tauri 2 provides the pieces for a background tray app:
- `WindowEvent::CloseRequested` with `CloseRequestApi::prevent_close`.
- `RunEvent::ExitRequested { code, api }` with `ExitRequestApi::prevent_exit`. `code` is `None` when the exit comes from user interaction, such as the last window closing, and `Some` when it comes from `AppHandle::exit` or `restart`.
- On macOS, `set_activation_policy(ActivationPolicy::Accessory)` and `set_dock_visibility(false)`.

## Method
Read `crates/tauri/src/app.rs` in the 2.11.5 tree.

## Result
- `prevent_exit` is ignored for restarts.
- The activation policy defaults to `NSApplicationActivationPolicyRegular`.

## Evidence tier
1.

## Performance
Not applicable.

## Correctness
API semantics come from the doc comments and code. Windows and Linux have no activation policy. Hiding from the taskbar there uses window APIs (`skip_taskbar`), not examined here.

## Relevance to hyperpace
- On close, hide or destroy the window.
- Keep the app alive by calling `prevent_exit` only when `code` is `None`.
- The tray "Quit" calls `app.exit(0)`, which yields `Some(0)` and is allowed through.
- On macOS, use Accessory for menu-bar-only mode, possibly switching to Regular while the settings window is open (behaviour to verify).
