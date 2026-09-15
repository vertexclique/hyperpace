# SUMMARY: Apple forum 696673, IOHIDRequestAccess does not update in real time

## Claim
- Changing an app's Input Monitoring permission while it runs does not take effect until the app relaunches; macOS offers "Quit & Reopen".
- IOHIDCheckAccess does not reflect the change.
- DTS: TCC is complex and buggy at times; sign with a stable designated requirement so build N+1 keeps the permission granted to build N.

## Method
Developer forum thread with community answers and a DTS reply.

## Result
- Permission changes need a relaunch.
- Unsigned or ad hoc-signed dev builds may lose or confuse TCC grants between builds.

## Evidence tier
2 (DTS reply), 3 (community observations).

## Performance
Not applicable.

## Correctness
- The relaunch behavior is observed by users (2021 to 2022) and may have changed since.
- The stable designated-requirement advice is DTS guidance.

## Relevance to hyperpace
- If Input Monitoring turns out to be needed, the macOS flow must say "grant, then relaunch".
- Release builds must be signed with a stable Developer ID so grants survive updates.
