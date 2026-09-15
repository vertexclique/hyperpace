URL: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/app.rs
Additional URL: https://docs.rs/tauri/2.11.5/tauri/enum.RunEvent.html
Retrieved: 2026-09-15 (shallow clone of `dev`, commit bca4ca58da02f182ef00ef1165e40e400a3cd8dd; crate version in tree 2.11.5)
Source type: framework source code (primary)

## Verbatim, ExitRequestApi and CloseRequestApi (app.rs lines ~78-105)

```rust
/// Api exposed on the `ExitRequested` event.
#[derive(Debug, Clone)]
pub struct ExitRequestApi {
  tx: Sender<ExitRequestedEventAction>,
  code: Option<i32>,
}

impl ExitRequestApi {
  /// Prevents the app from exiting.
  ///
  /// **Note:** This is ignored when using [`AppHandle#method.restart`].
  pub fn prevent_exit(&self) {
    if self.code != Some(RESTART_EXIT_CODE) {
      self.tx.send(ExitRequestedEventAction::Prevent).unwrap();
    }
  }
}

/// Api exposed on the `CloseRequested` event.
#[derive(Debug, Clone)]
pub struct CloseRequestApi(Sender<bool>);

impl CloseRequestApi {
  /// Prevents the window from being closed.
  pub fn prevent_close(&self) {
    self.0.send(true).unwrap();
  }
}
```

## Verbatim, RunEvent::ExitRequested (app.rs lines ~215-235)

```rust
pub enum RunEvent {
  /// Event loop is exiting.
  Exit,
  /// The app is about to exit
  #[non_exhaustive]
  ExitRequested {
    /// Exit code.
    /// [`Option::None`] when the exit is requested by user interaction,
    /// [`Option::Some`] when requested programmatically via [`AppHandle#method.exit`] and [`AppHandle#method.restart`].
    code: Option<i32>,
    /// Event API
    api: ExitRequestApi,
  },
```

## Verbatim, App::run doc example (app.rs lines ~1355-1370)

```rust
  /// app.run(|_app_handle, event| match event {
  ///   tauri::RunEvent::ExitRequested { api, .. } => {
  ///     api.prevent_exit();
  ///   }
  ///   _ => {}
  /// });
```

## Verbatim, macOS activation policy and dock visibility (app.rs lines ~625-665)

```rust
  /// Sets the activation policy for the application. It is set to `NSApplicationActivationPolicyRegular` by default.
  ///
  /// # Examples
  /// ```,no_run
  /// tauri::Builder::default()
  ///   .setup(move |app| {
  ///     #[cfg(target_os = "macos")]
  ///     app.handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
  ///     Ok(())
  ///   });
  /// ```
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  pub fn set_activation_policy(&self, activation_policy: ActivationPolicy) -> crate::Result<()> {
```

```rust
  /// Sets the dock visibility for the application.
  ///
  /// # Examples
  /// ```,no_run
  /// tauri::Builder::default()
  ///   .setup(move |app| {
  ///     #[cfg(target_os = "macos")]
  ///     app.handle().set_dock_visibility(false);
  ///     Ok(())
  ///   });
  /// ```
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  pub fn set_dock_visibility(&self, visible: bool) -> crate::Result<()> {
```
