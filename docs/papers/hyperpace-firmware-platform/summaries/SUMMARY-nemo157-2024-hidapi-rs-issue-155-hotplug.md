# SUMMARY: hidapi-rs issue 155, no device monitoring API

## Claim
hidapi-rs cannot notify about device arrival or removal. Users re-enumerate periodically. Prototype branches exist for the linux-native backend (udev monitor) and the C hidraw backend (hidapi's connection-callback branch), but nothing is merged.

## Method
GitHub issue thread, plus a grep of the 2.6.7 public API for any monitor or hotplug function.

## Result
- The issue was still open on 2026-09-15, with no maintainer reply in the thread.
- The 2.6.7 public API has no watch or monitor function.
- A commenter notes that hidapi enumeration "goes through a full bus enumeration".
- The prototype author reports enumeration-versus-monitor race conditions he calls unavoidable.

## Evidence tier
3 (community thread), plus 1 for the API check (the code has no such function).

## Performance
- Polling re-enumeration adds latency equal to the poll interval and repeats a full enumeration each time.
- No measured cost is available.

## Correctness
- Proven: the API is absent.
- The claim that races are unavoidable holds only for a separate enumerate-then-watch design. The documented Windows pattern (register first, then list) closes the gap at the cost of duplicate events.

## Relevance to hyperpace
- Pairing hidapi-rs with a separate hotplug source is required.
- Order: start the watcher first, then enumerate, and dedupe by path.
