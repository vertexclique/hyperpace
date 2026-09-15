# SUMMARY: HIDAPI issue 503, sporadic IOHIDDeviceSetReport general error on macOS

## Claim
About 5% of identical 64-byte output reports failed with IOHIDDeviceSetReport 0xE00002BC (general error), with no multithreading involved, yet the device received and answered them. It appeared with newer SDK builds. No root cause was recorded.

## Method
User report with maintainer triage.

## Result
- The failure was intermittent, and the device still acted on the command.
- The maintainer judged "there is no actual error".
- The issue was closed without a documented fix.

## Evidence tier
2/3 (maintainer triage of a user report).

## Performance
A 5% false-error rate on writes in that user's setup.

## Correctness
- Unknown cause.
- Not reproduced by others in the thread.
- May be device-, host- or SDK-specific.

## Relevance to hyperpace
- On macOS, do not treat one SetReport error as a lost command. The request/response matcher should wait out the response window (~200 ms) before retrying, and retries must be safe to repeat.
- Also relevant to write-flash commands: re-read after a failed write before rewriting.
