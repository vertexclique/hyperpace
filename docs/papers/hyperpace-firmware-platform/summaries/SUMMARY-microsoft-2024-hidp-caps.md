# SUMMARY: Microsoft, HIDP_CAPS structure

## Claim
HIDP_CAPS describes a single top-level collection. Its Input, Output and FeatureReportByteLength fields hold the largest report of each type in that TLC, counting the report-ID byte. UsagePage and Usage are the TLC's own usage pair.

## Method
Official WDK structure reference.

## Result
- A TLC with output report 8 (16 data bytes) and output report 9 (48 data bytes) has OutputReportByteLength 48 + 1 = 49.
- A TLC with only report 8 has OutputReportByteLength 16 + 1 = 17.
- InputReportByteLength follows the same rule.
- Windows, and hidapi on Windows, report the UsagePage/Usage pair per TLC path.

## Evidence tier
1 (official documentation). The 49 and 17 figures are arithmetic from that definition, not measurements.

## Performance
Not applicable.

## Correctness
- Proven: each field is a per-TLC maximum, and the report ID adds one byte.
- Assumed: whether HYPACE reports 8 and 9 sit in the same TLC or in different ones. The vendor interface declares usage pages 0x01, 0x0C and 0xFF02..0xFF06, so it almost certainly has several TLCs.
- A sibling device from the same manufacturer (Lamzu Atlantis, VID 0x3554) keeps report 8 alone in its 0xFF02/0x0002 TLC and has no report 9 (see SUMMARY-openmouse-2026-lamzu-atlantis-testing).
- Where HYPACE report 9 sits is unverified. A report-descriptor dump settles it.

## Relevance to hyperpace
- The Windows write length for report 8 is not a protocol constant. It is the `OutputReportByteLength` of whichever TLC path is opened. Read it from the handle (hidapi does this internally); do not hardcode 17 or 49.
- UsagePage/Usage from HIDP_CAPS is how to pick the report-8 path on Windows.
