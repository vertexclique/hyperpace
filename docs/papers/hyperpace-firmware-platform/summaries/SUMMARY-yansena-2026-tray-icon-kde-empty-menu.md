# SUMMARY: yansena-2026-tray-icon-kde-empty-menu

## Claim
On KDE Plasma 6.7.1 Wayland (Fedora 44), a tray icon built without a menu never appeared; assigning an empty `Menu` made it show. Maintainers closed the issue and said they plan to replace AppIndicator with ksni.

## Method
A user report, a maintainer attempt to reproduce, and a resolution by another user citing the documented quirk.

## Result
Fixed by setting a menu; no code change.

## Evidence tier
3 (report and fix), 2 (maintainer statements).

## Performance
Not applicable.

## Correctness
Single environment. The documented quirk ("Sometimes the icon won't be visible unless a menu is set") corroborates it.

## Relevance to hyperpace
Always attach a menu to the tray icon on Linux. It also confirms the maintainers' ksni direction.
