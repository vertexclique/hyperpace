# SUMMARY: opinionate-2026-notification-windows-permission-denied

## Claim
On Windows, the plugin's injected init script forces `window.Notification.permission` to "denied" on every page load, although the backend always reports Granted.

## Method
The reporter read the guest JS code. Two open PRs remove the Windows short-circuit.

## Result
Unfixed on the `v2` branch at retrieval.

## Evidence tier
3 (report), with PR discussion.

## Performance
Not applicable.

## Correctness
The quoted code path matches the plugin's guest-js. Not executed by this stream.

## Relevance to hyperpace
Do not gate notifications on the JS permission API; send them from Rust.
