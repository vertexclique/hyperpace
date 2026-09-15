# SUMMARY: Apple CoreHID framework

## Claim
CoreHID (macOS 15.0+) is Apple's modern HID framework:
- HIDDeviceManager for discovery;
- HIDDeviceClient for sending requests and receiving device data;
- HIDVirtualDevice for emulation.

Apple DTS recommends it over IOKit HID in recent forum answers.

## Method
Documentation index JSON via an extraction tool.

## Result
- macOS 15.0 minimum. Swift API.
- The retrieved index contains no permission or entitlement text.
- No Rust binding was searched for.

## Evidence tier
1 (official documentation index).

## Performance
Not evaluated.

## Correctness
- Proven: existence and availability.
- Unknown: its permission model for vendor devices, its behavior versus IOKit, and the effort of calling it from Rust (Swift bridge needed).

## Relevance to hyperpace
- Not needed if IOKit via hidapi works.
- A macOS 15+ only CoreHID backend would exclude older supported macOS versions (an operator decision) and needs a Swift/ObjC bridge.
- Keep as a fallback if IOKit HID access degrades in future macOS releases.
