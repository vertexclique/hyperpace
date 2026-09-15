URL: https://developer.apple.com/forums/thread/724608
Retrieved: 2026-09-15
Source type: Apple Developer Forums thread with Apple DTS engineer replies (Quinn), February 2023.
Retrieval method: WebFetch (extraction model); quoted sentences were returned in quotation marks.
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Title: "App with sandbox TCC deny IOHIDDeviceOpen"

Question:
> "Hi, I am developing macOS app with sandbox. I am implementing HID Device functionality in macOS app which shows error.
>
> 0x1000638ee: TCC deny IOHIDDeviceOpen
>
> Could not open HID manager
>
> In entitlements : com.apple.security.device.usb is set to true"

Clarification from the asker: "I am counting number of keys pressed by the user."

DTS engineer:
> "You don't need HID for that. Rather, check out `CGEventTap`. ...
>
> This API requires the System Settings > Privacy & Security > Input Monitoring privilege. CG has some little known but super useful routines to help with that: `CGPreflightListenEventAccess` and `CGRequestListenEventAccess`. And because this requires explicit user approval, it's supported even in sandboxed apps."
