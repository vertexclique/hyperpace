URL: https://github.com/tauri-apps/tray-icon/issues/336
Retrieved: 2026-09-15 (via `gh issue view 336 --repo tauri-apps/tray-icon`)
Source type: GitHub issue (community report, maintainer triage); state CLOSED (2026-09-04)

Title: "Tray icon never registers with StatusNotifierWatcher on KDE Plasma 6 / Wayland (libayatana-appindicator)"

## Verbatim, body (yansena)

> On KDE Plasma 6.7.1 (Wayland session, Fedora 44), a `TrayIconBuilder`-created tray icon never shows up in the system tray, even though the underlying `libayatana-appindicator` object is created and reachable on the session bus.

## Verbatim, maintainer comment (FabianLars, 2026-07-07)

> hmm, i can't reproduce this on fedora kde 44 wayland, tray icon shows up fine 🤔 not too familiar with that part of linux so idk how to debug this. maybeee we should ignore it since we'll be switching out appindicator for ksni soon anyway.

## Verbatim, resolution (dw-0, 2026-09-04)

> Turns out I was able to get the tray icon working on Fedora 44 / KDE Plasma / Wayland by assigning an empty `Menu` to the `TrayIcon`.
>
> The Tauri API documentation mentions this Linux-specific behavior:
>
> > "Sometimes the icon won't be visible unless a menu is set. Setting an empty `Menu` is enough."

## Verbatim, maintainer close (FabianLars, 2026-09-04)

> Oh damn, i completely missed that in the reproduction code above, my bad!! Since the tray-icon docs mention this as well, i'll close this.
