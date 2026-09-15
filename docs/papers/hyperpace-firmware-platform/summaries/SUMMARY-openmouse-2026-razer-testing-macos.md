# SUMMARY: OpenMouse Razer testing notes, macOS Input Monitoring on a mouse control channel

## Claim
Razer's control channel sits on a Generic Desktop Mouse collection. On macOS the browser's IOHIDDeviceOpen is refused ("Failed to open the device") unless Input Monitoring is granted and the browser relaunched, while the device still shows in the picker. Confirmed on a DeathAdder V2. Vendor software (Synapse) holding the interface also makes reads time out.

## Method
Community hardware testing checklist (tier-3 real-device observations).

## Result
- A configuration channel inside a mouse-conforming node requires Input Monitoring.
- The symptom looks like a driver bug: the device is enumerated but the open fails.
- Competing vendor software is the most common cause of failures.

## Evidence tier
3 (community hardware observation), consistent with tier-1 Apple source.

## Performance
Not applicable.

## Correctness
- Matches IOHIDFamily: a node that conforms to GD Mouse gets kIOHIDRequiresTCCAuthorizationKey.
- The observations come through Chrome, whose own open path may differ from hidapi, but the IOKit gate is the same.

## Relevance to hyperpace
- If any HYPACE vendor-node usage pair is a mouse or keyboard usage, the macOS app needs a first-run Input Monitoring flow with relaunch.
- Detect and report Lofree's configurator or a browser tab holding the device, where the OS makes that possible.
