# SUMMARY: giovannifranco1-2025-notification-gnome46-missing

## Claim
Notifications from the plugin do not appear on GNOME 46 and later (the report covers Fedora 41, Ubuntu 24.04 and GNOME 47). Per open PR 3588, GNOME closes a freedesktop notification when the sender's D-Bus unique name disappears. The plugin drops the notify-rust handle, and with it the connection, right after sending.

## Method
A user report, open since 2025-03-24. A fix PR opened 2026-09-15 holds the handle via `on_close`. It was tested manually on Arch GNOME 50 (Wayland and X11) and macOS.

## Result
The PR was unmerged at retrieval. Notifications still vanish when the app process exits, which is GNOME behaviour.

## Evidence tier
3 (report and community PR; no maintainer review yet).

## Performance
Holding a handle keeps one D-Bus connection open per notification.

## Correctness
The root-cause explanation is the PR author's, tested on GNOME 50 only.

## Relevance to hyperpace
If the operator's CachyOS desktop or target users run GNOME, low-battery notifications through plugin 2.4.0 may be invisible. Options: wait for the fix, or use notify-rust directly from Rust and keep the handle alive until the notification closes. hyperpace stays running in the tray, so process exit is not the problem.
