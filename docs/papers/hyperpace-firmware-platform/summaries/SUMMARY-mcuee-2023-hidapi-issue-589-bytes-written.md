# SUMMARY: HIDAPI issue 589, documenting bytes written and read

## Claim
The number of bytes HIDAPI reports for writes and reads is inconsistent across platforms and poorly documented. The issue asks for documentation, not code changes.

## Method
Issue opened by a HIDAPI collaborator, referencing earlier discussion in issue 478 and a CircuitPython test device (64-byte reports, no report IDs).

## Result
- Open since 2023-06-16.
- The verbatim body establishes that the discrepancy exists and points to issue 478 comments for specifics.
- The exact Windows figure (65 versus 64) came only from an extraction-tool paraphrase and is unverified.

## Evidence tier
2 (collaborator issue), weak on specifics.

## Performance
Not applicable.

## Correctness
- Proven: the ambiguity is acknowledged upstream and still open.
- The per-platform numbers were not verified verbatim. The concrete Windows over-count is instead proven by the source (SUMMARY-jefdriesen-2017-hidapi-issue-352-write-return and the C source).

## Relevance to hyperpace
Treat write return values only as success or failure. Confirm delivery through the report-8 response, not the byte count.
