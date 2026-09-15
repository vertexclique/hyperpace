URL: https://github.com/tauri-apps/tray-icon/issues/177
Retrieved: 2026-09-15 (via `gh issue view 177 --repo tauri-apps/tray-icon`)
Source type: GitHub issue (community report with a maintainer reply); state OPEN; opened 2024-07-13

Title: "support for CentOS9 / RHEL9?"

## Verbatim, body (bryanlarsen)

> tray-icon does not create tray icons on RHEL or CentOS stream 9.    Not surprising, they use a version of GNOME that doesn't support tray icons out of the box, so at first glance this seems like an obvious WONTFIX.

> Installation of https://extensions.gnome.org/extension/615/appindicator-support/ allows the slack tray icon to show and work.

> tray-icon works fine on Fedora Core 39 and later, so RHEL10 will likely work fine, but RHEL10 is still a year away.

## Verbatim, maintainer reply (amrbashir, 2024-07-17)

> We use `libappindicator-rs` under the hood which tries to use `libayatana-appindicator` if exists and fallsback to old `libappindicator` if it doesn't, I don't see why it shouldn't work but I don't use CentOS9/RHEL9 and really don't have the time to setup one to test so hopefully someone with access to these distros can help fix this.

## Verbatim, reporter follow-up (bryanlarsen, 2024-07-17)

> libappindicator-rs works for us on CentOS9.   It certainly could be PBKAC that prevents tray-icon from working for us on CentOS9.

## Related (web search result summary, tier 3, not independently verified by this stream)

A WebSearch for "Tauri system tray icon GNOME not showing AppIndicator extension required Linux" returned https://linuxiac.com/how-to-enable-system-tray-icons-in-gnome/ as a how-to for enabling tray icons in GNOME via the AppIndicator extension. The page itself was not fetched.
