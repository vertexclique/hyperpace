# SUMMARY: bryanlarsen-2024-tray-icon-gnome-appindicator

## Claim
On RHEL/CentOS Stream 9 (GNOME without tray support out of the box), tray-icon icons do not appear. Installing the GNOME "AppIndicator and KStatusNotifierItem Support" extension makes them show.

## Method
A user report comparing Slack, RustDesk and the reporter's app, plus a maintainer reply that the crate tries libayatana first, then libappindicator.

## Result
The issue is still open. The reporter later found that the underlying libappindicator-rs works on CentOS 9, and suspected their own usage.

## Evidence tier
3 (report), 2 (maintainer reply).

## Performance
Not applicable.

## Correctness
- Old GNOME 40 context.
- That stock GNOME Shell needs an extension to host AppIndicator/SNI icons was not verified here from GNOME's own docs.
- Which distributions ship the extension enabled by default was not verified.

## Relevance to hyperpace
On GNOME the tray may be invisible, so hyperpace must stay usable without it. Either do not hide the window on close unless a tray host is known to exist, or offer a setting. Whether a StatusNotifierWatcher can be detected at runtime was not verified (open question; ksni may expose this).
