URL: https://github.com/tauri-apps/plugins-workspace/issues/3512
Additional URLs (fix PRs, same thread): https://github.com/tauri-apps/plugins-workspace/pull/3540 ; https://github.com/tauri-apps/plugins-workspace/pull/3541
Retrieved: 2026-09-15 (via `gh issue view` / `gh pr view`)
Source type: GitHub issue (community root-cause analysis, OPEN since 2026-07-23) and two open PRs (one by a community contributor, one by a maintainer-side contributor)

## Verbatim, issue title and body (opinionate)

> [bug][notification] Windows: init script always sets `window.Notification.permission` to `"denied"` at startup

> On Windows, the notification plugin's injected init script stamps
> `window.Notification.permission = "denied"` on every page load, even though the
> desktop backend unconditionally reports `PermissionState::Granted` on Windows.
> Any app that reads `window.Notification.permission` (or gates sending on it)
> concludes notifications are blocked.

```ts
async function isPermissionGranted(): Promise<boolean> {
  if (window.Notification.permission !== 'default' || __TEMPLATE_windows__) {
    return await Promise.resolve(window.Notification.permission === 'granted')
  }
  return await invoke('plugin:notification|is_permission_granted')
}
```

## Verbatim, PR #3540 (state OPEN, 2026-08-17)

> The Windows short-circuit is removed, so Windows now falls through to `plugin:notification|is_permission_granted` like macOS and Linux. The desktop backend already reports `Granted`.

> Its recorded purpose was to avoid a Windows WebView `STATUS_ACCESS_VIOLATION` when loading remote websites.

## Verbatim, PR #3541 "fix(notification): always set permission granted on desktop" (state OPEN, 2026-08-18)

> Closes #3512
> Closes #3540
>
> I honestly think we should stop injecting to browser APIs at least not as defaults...
