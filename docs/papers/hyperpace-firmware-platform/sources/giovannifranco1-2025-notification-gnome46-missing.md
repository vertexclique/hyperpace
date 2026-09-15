URL: https://github.com/tauri-apps/plugins-workspace/issues/2566
Additional URL (fix PR, same thread): https://github.com/tauri-apps/plugins-workspace/pull/3588
Retrieved: 2026-09-15 (via `gh issue view` / `gh pr view`)
Source type: GitHub issue (community report, OPEN since 2025-03-24) and an open community fix PR (created 2026-09-15)

## Verbatim, issue title and body (giovannifranco1)

> [bug] no notifications in gnome 46+ (eg fedora 41 and ubuntu 24.04)

> I'm experiencing an issue where the plugin does not work on Fedora 41. The system is properly set up with GTK, and all dependencies seem to be in place. However, when I try to use the plugin, nothing happens: there are no logs or error messages to help diagnose the issue.

(Original punctuation: "nothing happens" followed by an em-dash, replaced here with a colon per project rule.)

> - Node: v20
> - Gnome 47

## Verbatim, PR #3588 "fix: keep GNOME notifications open (fix #2566)" (The-Best-Codes), state OPEN

> On GNOME (possibly 46 and later, but I haven't tested on all versions), the notification server closes an FDO notification once the sender's D-Bus unique name goes away. notify-rust keeps that name alive on the `NotificationHandle`.
>
> Before this PR, the plugin ran `let _ = notification.show()` inside a short-lived task, so the handle and its connection dropped right after the `Notify` call and the banner disappeared.
> Now the task holds the handle by calling `handle.on_close(...)`, which waits for the server's `NotificationClosed` signal.

> Note that notifications still go away when the app process exits, since that is GNOME's FDO behavior rather than the dropped handle.

> Manually tested on:
>
> - Arch Linux
>   - Gnome 50 using Wayland
>   - Gnome 50 using X11 (via `GDK_BACKEND=x11` env var)
> - macOS 27 (unaffected)
>
> I have **not** tested it on Windows but it should be unaffected.
