# SUMMARY: Apple forum 724608, "TCC deny IOHIDDeviceOpen" in a sandboxed app

## Claim
A sandboxed app opening a keyboard to count key presses gets "TCC deny IOHIDDeviceOpen" even with the com.apple.security.device.usb entitlement. DTS says to use CGEventTap for that purpose. It needs the Input Monitoring grant, which works even in sandboxed apps (CGPreflightListenEventAccess / CGRequestListenEventAccess).

## Method
Developer forum Q&A with DTS replies.

## Result
- The log string "TCC deny IOHIDDeviceOpen" is the IOHIDLib message emitted when the node requires TCC and access is not granted (matches IOHIDDeviceClass.m source).
- The USB entitlement does not bypass TCC.

## Evidence tier
2 (DTS forum replies), consistent with tier-1 source.

## Performance
Not applicable.

## Correctness
- Proven for keyboard devices.
- It says nothing about vendor-page devices.

## Relevance to hyperpace
- If macOS logs show "TCC deny IOHIDDeviceOpen" for the HYPACE vendor node, its usage-pair list contains keyboard, mouse or touchpad.
- Distribute outside the Mac App Store sandbox unless needed. Entitlements alone do not grant HID access to protected nodes.
