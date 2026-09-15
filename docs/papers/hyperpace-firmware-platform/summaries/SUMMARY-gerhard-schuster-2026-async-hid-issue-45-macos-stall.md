# SUMMARY: async-hid issue 45 and PR 46, macOS input delivery stops after SetReportWithCallback

## Claim
With one handle held open, async-hid 0.5.3 on macOS stops invoking the input report callback after somewhere between 30 and 444 `IOHIDDeviceSetReportWithCallback` writes. Reads then hang forever while writes keep succeeding. Replacing the call with synchronous `IOHIDDeviceSetReport` on a global dispatch queue removes the stall.

## Method
Contributor measurements on Apple silicon with one test board (Yepkit YKUSH3), one write and one read per round. Four alternative hypotheses were ruled out, and an A/B comparison was run against hidapi.

## Result
- Unpatched: nine runs stalled after 30, 39, 66, 70, 113, 232, 264, 269 and 444 exchanges. Throttling to 50/s does not avoid it ("a race, not a threshold").
- Patched: 86,151 and 81,984 exchanges in 180 s (479/s and 455/s), no stall, flat memory.
- hidapi on the same loop: 86,840 in 180 s (482/s).
- The PR also closes a soundness hazard: a caller buffer pointer was passed to an asynchronous IOKit call.
- Issue and PR were both open, with no maintainer response yet.

## Evidence tier
3 (careful contributor measurement, one device, not confirmed by the maintainer).

## Performance
About 480 exchanges/s ceiling on that board for both hidapi and the patched async-hid.

## Correctness
- Reproduced nine times by one reporter on one device model.
- The root cause inside IOKit is not explained.
- Whether HYPACE triggers it is unknown, but the precondition (a long-held handle with periodic write and read) matches hyperpace exactly.

## Relevance to hyperpace
- Do not use async-hid 0.5.3 unpatched for macOS device I/O.
- hidapi's synchronous SetReport path is the known-good reference.
