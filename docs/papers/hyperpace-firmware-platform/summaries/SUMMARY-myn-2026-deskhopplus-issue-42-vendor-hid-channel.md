# SUMMARY: deskhopplus issue 42, vendor HID channel design (macOS TCC and Windows path stability)

## Claim
- Measured: a vendor-defined usage page needs no permission on macOS and no driver on Windows.
- macOS `IOHIDDevice::conformsTo` walks the node's complete usage-pair list, so one keyboard collection anywhere in a node makes the whole node require Input Monitoring, vendor collections included.
- "Three usage pairs" are gated.
- Windows device interface paths embed a collection index and are not stable across reconnects, so locate devices by VID/PID, serial, usage page and usage.
- Interface disappearing and reappearing is normal operation.

## Method
Design spec in a hardware project, citing measurements.

## Result
- Two precise, checkable claims. The macOS one was verified against Apple source: three pairs, GD Keyboard, GD Mouse, Digitizer TouchPad; conformsTo iterates every pair.
- A practical identity rule for Windows reconnects.

## Evidence tier
3 (community design spec), with the macOS claim upgraded by tier-1 confirmation.

## Performance
Arithmetic only: about 64 KB/s per direction per channel at full speed. Not relevant to hyperpace's small reports.

## Correctness
- The macOS gating claim is confirmed by IOHIDFamily source.
- The Windows path-stability claim is from their observation and is consistent with container-ID documentation seen in search. Not independently verified here.

## Relevance to hyperpace
- Identify devices by (VID, PID, usage_page, usage), plus a serial or protocol-level identity if one exists. Never persist a Windows path.
- Receiver (FB16) and wired (FB14) are different USB devices. The "same physical mouse" identity has to come from the protocol, for example a device address, not the OS. That is an inference.
- The macOS permission outcome depends on the HYPACE vendor node's usage pairs.
