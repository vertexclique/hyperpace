# SUMMARY: OpenMouse Lamzu Atlantis capture, a VID 0x3554 sibling on Windows 11

## Claim
On Windows 11, the Lamzu Atlantis Mini 4K (VID 0x3554, the ODM's shared vendor ID) splits interface MI_01 into six TLCs:
- Col01 0xff05/0x00, Col02 0xff03/0x00: vendor, reject WriteFile with "Incorrect function".
- Col03 0x0c/0x01: consumer control.
- Col04 0x01/0x80: system control.
- Col05 0xff02/0x0002: the configuration channel, with report 8 in both directions.
- Col06 0xff04/0x02: feature report 6 only.

Report 8 is 17 bytes on the wire. Chrome's view shows input reports 16, 2, 5 and 8, feature report 6, and no report 9.

## Method
Community hardware capture on a real device (firmware 1.24, wired), cross-referenced with Chrome's `navigator.hid` collection dump and the vendor's Windows configurator config. Driver code gates on a collection with exactly one input and one output report, both ID 8, and matches responses by command byte.

## Result
- The vendor interface layout closely resembles HYPACE's description: pages 0x01, 0x0C and 0xFF02..0xFF06; input reports 2, 3, 5, 11, 16; feature 6. That points to the same ODM firmware family.
- On that device, report 8 lives alone in 0xFF02/0x0002, so its OutputReportByteLength would be 17. That is inferred; the doc gives the 17-byte wire length, not the caps value.
- Writing to the other vendor TLCs fails.
- The battery reply (command 0x04) carries percent, charging flag and millivolts, with a misleading length byte.

## Evidence tier
3 (community hardware capture, one device, Windows only).

## Performance
Not measured.

## Correctness
- Directly observed on the Lamzu device.
- Transfer to HYPACE is an assumption. HYPACE adds report 9 (48 B input and output) and page 0xFF06, which the Lamzu does not show, so HYPACE's report-9 placement may change OutputReportByteLength for the report-8 TLC.
- Chrome and Windows views of report IDs differ, per the doc.

## Relevance to hyperpace
- The Windows selection rule to try first: usage_page 0xFF02, usage 0x0002, then confirm by a successful report-8 exchange.
- Handle "Incorrect function" as "wrong collection" and try the next vendor TLC.
- The battery length-byte quirk is a warning for the protocol stream.
