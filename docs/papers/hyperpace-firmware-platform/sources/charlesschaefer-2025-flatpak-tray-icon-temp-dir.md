URL: https://github.com/tauri-apps/tauri/issues/13599
Retrieved: 2026-09-15 (via `gh issue view 13599 --repo tauri-apps/tauri`)
Source type: GitHub issue (community root-cause analysis); state CLOSED

Title: "[bug] App doesn't show Tray Icon when running under a flatpak package"

## Verbatim, body (charlesschaefer)

> The app works great on window and Linux (flat, deb and appimage). But when I try to build it for FLatpak, It is not showing the tray icon. There is some "unknown" icon there and the app is in the tray. But it can't display the correct icon.

## Verbatim, root cause (charlesschaefer, 2025-06-10)

> After almost one week trying to understand why tray-icons wasn't working well with Flatpak, I've finally figured it out.
>
> Tauri saves the tray icon to the disk. But it saves to $XDG_RUNTIME_DIR, that isn't accessible to the flatpak sandboxed app by default. So, we need to give the app this permission or change the path to where Tauri saves the icon (i.e. $XDG_CACHE_HOME).
>
> I've added this info in the flatpak bundling documentation: https://github.com/tauri-apps/tauri-docs/pull/3349.

## Corroborating source lines, tray-icon `src/platform_impl/gtk/mod.rs` (AppIndicator backend, tray-icon main branch cloned 2026-09-15)

```
28:        let (parent_path, icon_path) = temp_icon_path(attrs.temp_dir_path.as_ref(), &id, 0)?;
31:            icon.inner.write_to_png(&icon_path)?;
55:        let _ = std::fs::remove_file(&self.path);
60:            temp_icon_path(self.temp_dir_path.as_ref(), &self.id, self.counter)?;
63:            icon.inner.write_to_png(&icon_path)?;
133:            .unwrap_or_else(std::env::temp_dir)
138:    let icon_path = parent_path.join(format!("tray-icon-{}-{}.png", id.as_ref(), counter));
```
