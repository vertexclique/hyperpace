# SUMMARY: Apple forum 774149, IOHIDDeviceSetReport thread safety

## Claim
A DTS engineer says most IOKit calls are basically thread-safe, because the user client and kernel driver serialize access. APIs bound to a run loop should still be called on that run loop's thread, and calling SetReport from several threads "may work" but should be avoided, because it can break the accessory's state.

## Method
Developer forum Q&A with a DTS answer.

## Result
- Keep all interaction with one IOHIDDevice on one thread or queue.
- Driving device state from several threads is a design error even when nothing crashes.

## Evidence tier
2 (Apple DTS engineer post).

## Performance
Not applicable.

## Correctness
- Guidance, not a documented contract.
- Consistent with HIDAPI's per-device single-thread rule.

## Relevance to hyperpace
Supports the per-device owner loop: all SetReport calls and callback handling for a device go through one thread or queue, and UI commands queue into it.
