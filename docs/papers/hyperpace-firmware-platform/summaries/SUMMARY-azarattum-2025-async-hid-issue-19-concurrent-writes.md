# SUMMARY: async-hid issue 19, concurrent writes on Windows

## Claim
async-hid on Windows fails a second concurrent write with "Another write operation is in progress". The maintainer calls this an intentional stopgap and advises wrapping the writer in an async mutex; a planned `&mut self` API makes the constraint explicit on all platforms. The report ID as first byte is the intended convention everywhere.

## Method
Issue thread with maintainer responses.

## Result
- Serialize writes per device in application code.
- Always prefix the report ID.
- The maintainer considers the first-byte report-ID API poor and may change it, a possible future breaking change.

## Evidence tier
2 (maintainer comments).

## Performance
Not applicable.

## Correctness
- Proven: the behavior and the intent.
- Whether the `&mut self` redesign shipped in 0.5.x was not checked.

## Relevance to hyperpace
- The per-device owner-loop design already serializes writes, so this constraint costs nothing.
- Wrap any report-ID handling in one function so an upstream API change stays contained.
