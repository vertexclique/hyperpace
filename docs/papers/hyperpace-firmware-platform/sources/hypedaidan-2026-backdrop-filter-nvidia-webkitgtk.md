URL: https://github.com/tauri-apps/tauri/issues/14811
Additional URL: https://github.com/tauri-apps/tauri/issues/14040
Retrieved: 2026-09-15 (via `gh issue view`)
Source type: GitHub issues (community reports, tier 3; maintainer triage comments tier 2)

## Verbatim, #14811 (OPEN, 2026-01-23, HypedAidan) "[bug] backdrop-filter doesn't render in Ubuntu 24.04 LTS w/ visual abnormalities"

> Through my vigorous testing, I discovered the backdrop-filter among with several other visual abnormalities are present while using Nvidia Drivers.

```text
    - OS: Ubuntu 24.4.0 x86_64 (X64) (ubuntu-wayland on wayland)
    ✔ webkit2gtk-4.1: 2.50.4
    - tauri 🦀: 2.9.5
```

Reporter comment (2026-01-23):
> Will add this is a WebKit2GTK issue but still should be addressed as Tauri exclusively uses WebKit2GTK

## Verbatim, #14040 (CLOSED, 2025-08-19, KaldrArt) "[bug] ubuntu backdrop-filter not rendering"

> css property backdrop-filter not working in tauri window.

> I tried webkitgtk  Epiphany, it showed the same useragent, but it rendered correctly.

Maintainer (FabianLars, 2025-08-19):
> Did you try it with the `-webkit-` prefix? i saw some say that tailwind doesn't (always) apply those.
>
> Also, in your manual test, did you use a `var()` ? Those don't work in webkit for backdrop-filter for some reason. eg https://github.com/tailwindlabs/tailwindcss/issues/13844

Reporter:
> I think it maybe a problem with gtk3.
>
> Epiphany uses python and gtk4, it produces correct rend

Maintainer (FabianLars, 2025-08-19):
> possible. it works for others though so i assume this is one of the many hardware specific issues webkitgtk has.

## Search listing (gh search issues --repo tauri-apps/tauri "backdrop-filter", 2026-09-15)

```
6876 open 2023-05-06 [bug] transparent + backdrop-filter: blur() cannot be effective when move window
14811 open 2026-01-23 [bug] backdrop-filter doesn't render in Ubuntu 24.04 LTS w/ visual abnormalities
12804 open 2025-02-24 [bug] Backdrop blur not working when enabling window transparency, the behavior is not as expected.
15512 closed 2026-06-09 [bug] The backdrop-filter: blur property does not work when the window is transparent.
14040 closed 2025-08-19 [bug] ubuntu backdrop-filter not rendering
```
