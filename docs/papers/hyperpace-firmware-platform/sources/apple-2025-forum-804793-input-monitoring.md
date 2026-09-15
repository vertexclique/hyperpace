URL: https://developer.apple.com/forums/thread/804793
Retrieved: 2026-09-15
Source type: Apple Developer Forums thread with an Apple DTS engineer reply (Kevin Elliott, CoreOS/Hardware), October 2025.
Retrieval method: WebFetch (extraction model). Staff sentences below were returned in quotation marks; verify exact wording on the page before quoting externally.
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Title (as returned): "Mac Catalyst: IOHID InputReportCallback not firing, USBInterfaceOpen returns kIOReturnNotPermitted (0xe00002e2) for custom HID device"

Question summary (tool summary, not verbatim): a sandboxed Mac Catalyst app talking to a vendor-specific usage page HID device; enumeration, open and IOHIDDeviceSetReport succeed, but IOHIDDeviceRegisterInputReportCallback never fires, and IOUSBInterfaceOpen / IOUSBInterfaceOpenSeize return kIOReturnNotPermitted.

DTS engineer reply:
> "I'd need to go look at the API again, but I think most of the other APIs are actually just retrieving data from the IORegistry, which doesn't really require any 'access' to the device itself. In terms of what it's failing, I suspect it's because you haven't authorized Input Monitoring, as described here."

> "We require 'Input Monitoring' for anything that the system itself 'directly' understands (keyboard, mice, etc) but I think you could avoid that requirement with the right device type."

Recommendation (tool summary): migrate to the CoreHID API.
