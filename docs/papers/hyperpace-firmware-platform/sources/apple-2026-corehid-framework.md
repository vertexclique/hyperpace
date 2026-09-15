URL: https://developer.apple.com/documentation/corehid (data endpoint https://developer.apple.com/tutorials/data/documentation/corehid.json)
Retrieved: 2026-09-15
Source type: Official Apple framework documentation index.
Retrieval method: WebFetch of the documentation JSON (extraction model; abstracts were returned in quotation marks).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Abstract:
> "Interact with keyboards, mice, and other human interface devices."

Overview:
> "The CoreHID framework facilitates interaction with human interface devices (HID), like a keyboard, mouse, or other device. Interactions include receiving data that a device generates, such as a key press or mouse click. CoreHID also allows sending requests to a device, such as a request to turn on an LED. You can also emulate a device connected to the system, such as a virtual game controller, and send input to other apps without physical hardware."

Availability (as returned): macOS, minimum version 15.0.

Symbols (abstracts as returned):
- HIDDeviceManager: "A helper for discovering human interface devices (HID) connected to the system."
- HIDDeviceClient: "A client of a physical or virtual HID compatible peripheral."
- HIDVirtualDevice: "A virtual service to emulate a HID device connected to the system."

Extraction note (tool summary): "No entitlement or permission requirement text is included in the provided documentation." The CoreHID API is Swift; no Rust binding was searched for in this stream.
