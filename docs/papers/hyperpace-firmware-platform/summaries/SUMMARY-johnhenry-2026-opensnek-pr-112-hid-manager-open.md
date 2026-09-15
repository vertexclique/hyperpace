# SUMMARY: OpenSnek PR 112, IOHIDManagerOpen refusal is not always a TCC denial

## Claim
In a macOS Razer tool, IOHIDManagerOpen over many devices returned kIOReturnNotPermitted even with Input Monitoring granted, triggered by "protected keyboard input interfaces". Per-device IOHIDDeviceOpen still succeeded. The fix uses IOHIDCheckAccess to tell a real denial from a structural refusal, and stops recreating the manager each cycle. Verified on macOS 15 with Razer keyboards.

## Method
Merged pull request description (via an extraction tool).

## Result
- A bulk manager open is fragile when protected devices are attached.
- Per-device open is the robust path.
- kIOReturnNotPermitted alone does not prove the app lacks permission.

## Evidence tier
3 (third-party project PR).

## Performance
Reduced repeated errors "to a single startup warning". No other numbers.

## Correctness
- Observed on specific Razer hardware on macOS 15.
- The mechanism behind "structural refusal" is not explained.

## Relevance to hyperpace
- hidapi does not call IOHIDManagerOpen; it copies devices and opens per device, which avoids this failure mode. A native backend should do the same.
- Error handling should check IOHIDCheckAccess before telling the user to grant Input Monitoring.
