URL: https://github.com/sidit77/async-hid/pull/48
Retrieved: 2026-09-15
Source type: GitHub pull request (contributor), OPEN at retrieval, with maintainer acknowledgement.
Retrieval method: `gh pr view 48 -R sidit77/async-hid --json ...` (verbatim).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`fix(win32): recover from interrupted HID writes | safonin | 2026-09-05T10:00:04Z | OPEN`

> - Treat `ERROR_IO_INCOMPLETE` as the only result that keeps an overlapped operation pending.
> - Clear the pending state after successful completion and terminal errors.
> - Propagate errors from abandoned writes and feature writes before starting another I/O operation.
> - Map `ERROR_NO_SUCH_DEVICE` to `HidError::Disconnected`.

> When a device disappears while an overlapped HID write is pending, `GetOverlappedResult` can return `0x800701B1` (`ERROR_NO_SUCH_DEVICE`), as reported in AprilNEA/OpenLogi#1200.
>
> A competing or failed receiver access has also produced `0x8007001F` (`ERROR_GEN_FAILURE`), as observed in this OpenLogi hardware report.
>
> Previously, these terminal errors were returned without clearing the internal `pending` flag. The write paths then swallowed the completion error and attempted to start another operation, triggering the `I/O operation already pending` assertion.

> Windows compilation and linting pass, but runtime Windows hardware validation was not performed because I do not currently have access to a Windows machine.

sidit77, 2026-09-06:
> Thanks for the PR. I am going to look at it once I am back from vacation. Sorry for the delay.
