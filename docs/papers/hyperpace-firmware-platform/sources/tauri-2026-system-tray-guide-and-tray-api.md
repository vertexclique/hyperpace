URL: https://v2.tauri.app/learn/system-tray/
Additional URLs: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/learn/system-tray.mdx (commit a6b59b7819f34edfacf365ce15fcd53c9f00ba11, 2026-09-13) ; https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/tray/mod.rs (commit bca4ca58, 2026-09-15) ; https://docs.rs/tauri/latest/tauri/tray/struct.TrayIconBuilder.html
Retrieved: 2026-09-15
Source type: official documentation and framework source (primary)

## Verbatim, docs: enabling the tray

```toml title="src-tauri/Cargo.toml"
tauri = { version = "2.0.0", features = [ "tray-icon" ] }
```

## Verbatim, docs: menu behaviour

> :::note
> By default the menu is displayed on both left and right clicks.
>
> To prevent the menu from popping up on left click, call the [`show_menu_on_left_click(false)`][TrayIconBuilder::show_menu_on_left_click] Rust function
> or set the [`menuOnLeftClick`] JavaScript option to `false`.
> :::

## Verbatim, docs: tray events and the Linux limitation

> The tray icon emits events for the following mouse events:
>
> - click: triggered when the cursor receives a single left, right or middle click, including information on whether the mouse press was released or not
> - Double click: triggered when the cursor receives a double left, right or middle click
> - Enter: triggered when the cursor enters the tray icon area
> - Move: triggered when the cursor moves around the tray icon area
> - Leave: triggered when the cursor leaves the tray icon area
>
> :::note
> Linux: Unsupported. The event is not emitted even though the icon is shown and will still show a context menu on right click.
> :::

## Verbatim, docs: Rust example (show main window on left click)

```rust
TrayIconBuilder::new()
  .on_tray_icon_event(|tray, event| match event {
    TrayIconEvent::Click {
      button: MouseButton::Left,
      button_state: MouseButtonState::Up,
      ..
    } => {
      println!("left click pressed and released");
      // in this example, let's show and focus the main window when the tray is clicked
      let app = tray.app_handle();
      if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
      }
    }
```

## Verbatim, framework source crates/tauri/src/tray/mod.rs (runtime updates)

```rust
  /// Sets a new tray icon. If `None` is provided, it will remove the icon.
  pub fn set_icon(&self, icon: Option<Image<'_>>) -> crate::Result<()> {
```

```rust
  /// Sets a new tray menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux**: once a menu is set it cannot be removed so `None` has no effect
  pub fn set_menu<M: ContextMenu + 'static>(&self, menu: Option<M>) -> crate::Result<()> {
```

```rust
  /// Sets the tooltip for this tray icon.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported
  pub fn set_tooltip<S: AsRef<str>>(&self, tooltip: Option<S>) -> crate::Result<()> {
```

```rust
  /// ## Platform-specific:
  ///
  /// - **Linux:** The title will not be shown unless there is an icon
  ///   as well.  The title is useful for numerical and other frequently
  ///   updated information.  In general, it shouldn't be shown unless a
  ///   user requests it as it can take up a significant amount of space
  ///   on the user's panel.  This may not be shown in all visualizations.
  /// - **Windows:** Unsupported
  pub fn set_title<S: AsRef<str>>(&self, title: Option<S>) -> crate::Result<()> {
```

```rust
  /// Disable or enable showing the tray menu on left click.
  ///
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux**: Unsupported.
  pub fn set_show_menu_on_left_click(&self, #[allow(unused)] enable: bool) -> crate::Result<()> {
```

`set_icon` body dispatches to the main thread: `run_item_main_thread!(self, |self_: Self| self_.inner.set_icon(icon))?.map_err(Into::into)`
