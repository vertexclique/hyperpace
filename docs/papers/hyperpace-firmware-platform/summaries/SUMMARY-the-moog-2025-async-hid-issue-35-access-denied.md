# SUMMARY: async-hid issue 35, Access denied opening a keyboard-class device on Windows

## Claim
Opening a keyboard TLC (usage 0x01/0x06) read/write on Windows 11 fails with E_ACCESSDENIED because Windows opens keyboards exclusively. The maintainer points to Microsoft's system-use TLC list. The reporter says a mouse opened fine despite being on the exclusive list.

## Method
User report plus maintainer explanation.

## Result
- The keyboard TLC is not openable read/write.
- The mouse observation contradicts the table. It is plausibly a different TLC of that mouse being opened, but unverified.

## Evidence tier
2 (maintainer), 3 (user observation).

## Performance
Not applicable.

## Correctness
- The keyboard case matches Microsoft docs.
- The mouse anecdote is unexplained, so do not rely on it.

## Relevance to hyperpace
- Enumeration must select the vendor TLC by usage page. Never open the boot-keyboard or mouse TLCs of 0x3554:FB16/FB14.
- The UI should explain an access-denied error as "wrong collection or another app holds it", not as a missing admin right.
