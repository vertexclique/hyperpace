URL: https://github.com/sidit77/async-hid/issues/45 ; https://github.com/sidit77/async-hid/pull/46
Retrieved: 2026-09-15
Source type: GitHub issue with measurements and the linked open pull request (contributor, not maintainer). Both OPEN at retrieval.
Retrieval method: `gh issue view 45 -R sidit77/async-hid --json ...` and `gh pr view 46 -R sidit77/async-hid --json ...` (verbatim, truncated).
Character note: em-dash characters in quoted text were replaced with ASCII hyphens; nothing else changed.

# Issue 45 excerpts

`macOS: SetReportWithCallback stops input report delivery after a few hundred writes | gerhard-schuster | 2026-08-17T00:44:48Z | OPEN`

> On macOS, when a device handle is held open across many exchanges,
> `read_input_report` stops completing. Every later read blocks forever - or times
> out, if you race a timer against it. Writes keep succeeding the whole time.
> Dropping the handle and opening the device again restores everything at once.
>
> The cause is the **write** path, not the read path. `write_report` uses
> `IOHIDDeviceSetReportWithCallback`; after a few hundred of those calls on one
> device, IOKit stops invoking the input report callback registered at open.
>
> Replacing that one call with the synchronous `IOHIDDeviceSetReport` - which is
> what hidapi's macOS backend uses - removes the failure entirely.
>
> Measured against `async-hid 0.5.3`. The same code is on `main` as of `1d34435`

> macOS on Apple silicon, a Yepkit YKUSH3 (`04D8:F11B`), one handle held open, one
> write and one read per round:
>
> | | exchanges | read timeouts |
> |---|---:|---:|
> | `async-hid 0.5.3` | 232 in 30 s, then silent | 10 |
> | with the patch below | **88 508 in 180 s** | **0** |

> Unpatched, the same loop stalls after somewhere between 66 and 444 exchanges.

Follow-up, 2026-08-17 (ruled-out hypotheses table):
> | The shared serial dispatch queue starves input report delivery | `DispatchQueueAttr::SERIAL` changed to `concurrent()` | **No.** Stalls identically, 127 rounds |

# PR 46 excerpts

`Do not use SetReportWithCallback on macOS | gerhard-schuster | 2026-08-18T15:14:46Z | OPEN`

> On macOS a device stops delivering input reports after a few hundred calls to
> `IOHIDDeviceSetReportWithCallback`. Reads on that handle then never complete
> again while writes keep succeeding, and only reopening the device restores it.
> A command line tool that opens, exchanges once and exits never notices. A
> process holding a device does, within the first minute.

> | | exchanges | stall | rate |
> |---|---:|---|---:|
> | `main` as it stands | 30, 39, 70, 269 | yes, permanent | |
> | this branch | 86 151 in 180 s | none | 479/s |
> | this branch, second run | 81 984 in 180 s | none | 455/s |
> | hidapi, same loop, same board | 86 840 in 180 s | none | 482/s |
>
> Nine runs of `main` have now stalled after 30, 39, 66, 70, 113, 232, 264, 269
> and 444 exchanges. It is a race, not a threshold: throttling to 50 exchanges
> per second does not avoid it.

> **It closes a soundness hazard.** The current code hands `data_to_send.as_ptr()`,
> a pointer into the caller's buffer, to an asynchronous IOKit call.
