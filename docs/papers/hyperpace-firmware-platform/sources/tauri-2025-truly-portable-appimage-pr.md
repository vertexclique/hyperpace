URL: https://github.com/tauri-apps/tauri/pull/12491
Additional URLs: https://github.com/tauri-apps/tauri/pull/14841 ; https://github.com/tauri-apps/tauri/pull/15917
Retrieved: 2026-09-15 (via `gh pr view` and `gh api repos/tauri-apps/tauri/issues/12491/comments`)
Source type: GitHub pull request by a Tauri core maintainer (FabianLars) with testing comments from the sharun/pkgforge AppImage tooling author (Samueru-sama) and a tester (zer0x64); tier 2

## PR metadata (gh pr view)

```
feat(bundler): Truly portable appimage (experimental) | OPEN | created 2025-01-24T00:39:49Z | updated 2026-09-07T07:01:18Z | merged null
```

## Verbatim, PR body

> will add description later

> fixes https://github.com/tauri-apps/tauri/issues/10617 (xdg-open)
> fixes https://github.com/tauri-apps/tauri/issues/10078 (xdg-open)
> fixes https://github.com/tauri-apps/tauri/issues/6172 (xdg-open)
> fixes https://github.com/tauri-apps/tauri/issues/14208 (NO_STRIP)

## Verbatim, PR #14841 body (merged 2026-01-28)

> resolving the conflicts in https://github.com/tauri-apps/tauri/pull/12491/ is killing me for no reason so i pulled out the reorga into a sep pr

## PR #15917 (merged 2026-08-25) title

"fix(bundler): give quick-sharun list of all bins/libs in AppDir. detect tray-icon feature."

## Verbatim, testing thread (2025-05-08)

zer0x64:
> Got even worse result, the app simply crash on launch now with error "Could not create GBM EGL display: EGL_SUCCESS. Aborting..."

zer0x64:
> Progress update: We narrowed it down to wayland, forcing it to use XWayland by setting an empty WAYLAND_DISPLAY seems to enable hardware accel (although the warnings are still there)

zer0x64:
> Ubuntu 24.04 (the latest LTS) uses an older mesa version that handle dri library loading differently. ...
> Sharun does not handle both ways at the same time. That means that to get hardware accel working on nvidia, you'd need to build on at least Ubuntu 25.04
> ...
> I think it is already a major improvement over the linuxdeploy-based build. For my case, I went from "not working at all on AMD" to "works on both GPUs, one is laggy unless you add an env variable"

FabianLars:
> i mean, if we have to use arch as well then so be it, i'm so done with fucking desktop linux and wanted to drop support as a whole for ages now so we may as well add another build restriction. 🤷

FabianLars:
> Ubuntu is not a hard requirement for us/tauri. I just assumed it was the best supported coming from other appimage tooling. I wanted to support arch (and ideally fedora) in the future anyway.
>
> Since users need 2 workflow jobs (one for appimage with latest ubuntu, one for deb/rpm for oldest) they may as well use an arch container for the appimage job.

Samueru-sama (2025-05-09):
> Arch is fine and perfect for building appimages, because arch doesn't split packages into a million pieces and puts all the libs in `/usr/lib`

Samueru-sama (2025-05-19):
> Alright [sharun now has a gbm hook](https://github.com/VHSgunzo/sharun/commit/3860033a659121b935b4f6740eb9489639ab2a0c), so it doesn't have to be explicitly passed to lib4bin anymore.

## Verbatim, maintainer pointer on issue #15665 (FabianLars, 2026-07-07)

> oh and you may want to try the new appimage bundler from https://github.com/tauri-apps/tauri/pull/12491 if you want to get actually working appimages - that requires building on arch though :/
