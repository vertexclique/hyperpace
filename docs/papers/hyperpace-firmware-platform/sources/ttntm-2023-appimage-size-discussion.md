URL: https://github.com/tauri-apps/tauri/discussions/6918
Retrieved: 2026-09-15 (via GitHub GraphQL API)
Source type: GitHub discussion with a Tauri core maintainer reply (tier 2) and a community size data point (tier 3); created 2023-05-09; no accepted answer

Title: "Any workaround to reduce AppImage size?"

## Verbatim, question

> I found most AppImage files much larger than other packages like those for Windows, MacOS, or even other Linux packages. I see this is mentioned in docs, but still I wonder if there is any chance to debloat it (I guess it's caused by webkit2gtk, which should be safe to remove for most distros). Otherwise, there is no big difference from an Electron version in package size.

## Verbatim, maintainer reply (FabianLars, 2023-05-09)

> No there is no way, well, i guess a few KB here and there in case the bundler includes a few things too eagerly.
>
> > I guess it's caused by webkit2gtk, which should be safe to remove for most distros
>
> Not at all, even though most Gnome based distros do have it installed you can't really expect this to be the case, because if it's not there the appimage can't handle that in contrast to something like the `.deb` bundle which installs it on the system if needed.
>
> At the end of the day this is just how appimages work, they are supposed to bundle everything they need to run and not doing this would cause more harm than good.

## Verbatim, community data point (ttntm, 2023-10-19)

> Just gave Tauri a shot and built an AppImage for my [time tracker](https://github.com/ttntm/itrack). It's a small Vue SPA - the web build amounts to 647.9 kB and the whole `node_modules` folder is 120.2 MB in size (36.2 MB of this is the folder `@tauri-apps`).
>
> The resulting AppImage is 89.3 MB which I find quite extreme TBH.

## Related (web search summaries, tier 3, pages not opened by this stream)

A WebSearch ("tauri v2 AppImage file size MB bundled webkit2gtk") surfaced tabularis.dev/blog/why-tabularis-runs-on-tauri and firezone.dev/blog/using-tauri; the search tool's summary attributed "~76 MB" AppImage vs "around 4 MB" .deb/.rpm, and "~10 MB to ~100 MB" when bundling the webview, to those pages. Not verified by fetching.
