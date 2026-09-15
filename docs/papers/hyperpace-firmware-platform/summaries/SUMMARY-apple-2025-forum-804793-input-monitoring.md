# SUMMARY: Apple forum 804793, DTS on Input Monitoring for custom HID devices

## Claim
An Apple DTS engineer (Kevin Elliott, October 2025) states that macOS requires Input Monitoring "for anything that the system itself 'directly' understands (keyboard, mice, etc)", and that the requirement can be avoided "with the right device type". In the questioner's case (vendor page, SetReport works, input callback silent, IOUSBInterfaceOpen returns kIOReturnNotPermitted), he suspected missing Input Monitoring and recommended CoreHID.

## Method
Developer forum Q&A with a DTS reply. Retrieved through an extraction tool, so wording must be verified before quoting.

## Result
- Gating is by device type, not by API.
- Registry reads (enumeration) need no access.
- The specific symptom (writes OK, no input) was only diagnosed speculatively.

## Evidence tier
2 (Apple DTS engineer forum post).

## Performance
Not applicable.

## Correctness
- The general rule is confirmed precisely by IOHIDFamily source: only keyboard, mouse and touchpad usage pairs set the TCC-required property.
- The DTS diagnosis of the questioner's vendor-page device was a guess ("I suspect"). Mac Catalyst and sandboxing were confounders.

## Relevance to hyperpace
- If HYPACE's vendor interface has no keyboard, mouse or touchpad usage pair, no Input Monitoring prompt should be needed.
- The UI should still detect kIOReturnNotPermitted and explain the Input Monitoring grant, in case the descriptor includes such a pair.
