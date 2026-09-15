URL: https://github.com/gh123man/OpenSnek/pull/112
Retrieved: 2026-09-15
Source type: GitHub pull request in a third-party macOS Razer configuration tool (merged August 27, 2026 per extraction).
Retrieval method: WebFetch (extraction model; the page was summarized, with short verbatim fragments in quotation marks).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Title: "Make HID-manager open failures non-fatal and correctly attributed"

Tool summary with quoted fragments:
- The probe "continues USB discovery when the manager-level open fails, since 'per-device `IOHIDDeviceOpen` can still succeed.'"
- "Previously, when macOS refused bulk manager open with `kIOReturnNotPermitted` despite Input Monitoring being granted (triggered by 'protected keyboard input interfaces'), the bridge incorrectly treated this as a TCC denial."
- "The solution uses `IOHIDCheckAccess` to differentiate states and reuses the manager when the refusal is structural rather than permission-based."
- "verified this on macOS 15 with Razer Huntsman Mini and Tartarus Pro devices"
