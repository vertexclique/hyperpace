URL: https://v2.tauri.app/distribute/pipelines/github/
Additional URLs: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/distribute/Pipelines/github.mdx (commit a6b59b78) ; https://github.com/tauri-apps/tauri-action/releases/tag/action-v1.0.0
Retrieved: 2026-09-15
Source type: official documentation and release notes (primary)

## Verbatim lines, docs workflow

```
67:6. Lastly, it uses `tauri-apps/tauri-action@v1` to run `tauri build`, generate the artifacts, and create a GitHub release.
86:          - platform: 'macos-latest' # for Arm based macs (M1 and above).
88:          - platform: 'macos-latest' # for Intel based macs.
90:          - platform: 'ubuntu-22.04'
92:          - platform: 'ubuntu-22.04-arm' # Only available in public repos.
94:          - platform: 'windows-latest'
105:          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf xdg-utils
117:          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}
129:      - uses: tauri-apps/tauri-action@v1
152:Github has [released](https://github.blog/changelog/2025-08-07-arm64-hosted-runners-for-public-repositories-are-now-generally-available/#get-started) publicly available `ubuntu-22.04-arm` and `ubuntu-24.04-arm` runners. You can use these to build your app for Arm64 in public repos with the workflow example above.
```

## tauri-action releases (gh api, newest three)

```
action-v1.0.0 2026-06-29T17:04:18Z
action-v0.6.2 2026-03-14T17:07:00Z
action-v0.6.1 2026-01-03T13:44:57Z
```

## Verbatim, action-v1.0.0 release notes (partial)

> - [`db4399e`] ([#1194] by [@FabianLars]) **Breaking Change**: `.app.tar.gz` & `.app.tar.gz.sig` files will now include the app version like all other bundles/installers.
> - [`53c8863`] ([#1315] by [@FabianLars]) The download urls in `latest.json` will now use the github url instead of the browser download url.
> - [`9b64567`] ([#1167] by [@FabianLars]) **Breaking Change**: Drop support for Tauri v1 and unstable v2 (alpha, beta, rc) versions.
> - [`9491f9d`] ([#1177] by [@FabianLars]) **Breaking Change**: The action will now fail if `draft: true` is set but the relevant release is not a draft.

## Verbatim, updater docs cross-reference (https://v2.tauri.app/plugin/updater/)

> [Tauri Action](https://github.com/tauri-apps/tauri-action) generates a static JSON file for you to use on CDNs such as GitHub Releases.
