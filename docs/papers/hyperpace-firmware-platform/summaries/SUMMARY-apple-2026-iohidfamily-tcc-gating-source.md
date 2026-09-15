# SUMMARY: Apple IOHIDFamily source, when macOS requires Input Monitoring for IOHIDDeviceOpen

## Claim
IOHIDFamily sets `kIOHIDRequiresTCCAuthorizationKey` on a device node only if `conformsTo` matches one of three pairs:
- GenericDesktop / Keyboard
- GenericDesktop / Mouse
- Digitizer / TouchPad

`conformsTo` walks every entry of the node's usage-pair list. On open, IOHIDLib's `initConnect` calls `IOHIDRequestAccess(kIOHIDRequestTypeListenEvent)` only when that property is true. If access is denied it logs "TCC deny IOHIDDeviceOpen" and returns kIOReturnNotPermitted; otherwise it grants access without a prompt. A separate "protected access" list applies only to Apple vendor IDs.

## Method
Read Apple's open-source IOHIDFamily (main, 2026-04-13).

## Result
- For HYPACE, each IOHIDDevice node (per USB interface) is judged by its full usage-pair list:
  - boot-keyboard interface (0x01, 0x07, 0x08): keyboard usage, likely gated (inferred from its keyboard usage);
  - mouse interface (0x01/0x02): gated;
  - vendor interface: gated only if one of its pairs is 0x01/0x06 or 0x01/0x02 (or a touchpad).
- If the vendor interface's page-0x01 usage is System Control (0x80), as on the same-vendor Lamzu sibling, and the 0x0C usage is Consumer Control, hyperpace needs no Input Monitoring. That is inferred.
- Keypad (0x01/0x07), Pointer (0x01/0x01), Consumer and System Control are not in the gated list.

## Evidence tier
1 (Apple source code).

## Performance
Not applicable. The TCC request happens once per IOHIDDevice client object (`_tccRequested`).

## Correctness
- Proven for this open-source drop.
- Assumed: that the shipping macOS 26.x uses the same logic.
- Assumed: that `newDeviceUsagePairs` includes every top-level collection's usage pair, which is consistent with hidapi enumerating `kIOHIDDeviceUsagePairsKey`.
- The HYPACE vendor node's actual usage pairs are unknown until a descriptor dump.

## Relevance to hyperpace
- This settles the macOS permission question at the mechanism level. Only a descriptor dump can confirm whether the vendor node conforms to keyboard or mouse.
- If it does not, the macOS UX needs no TCC flow. If it does, a first-run Input Monitoring flow with relaunch is mandatory.
