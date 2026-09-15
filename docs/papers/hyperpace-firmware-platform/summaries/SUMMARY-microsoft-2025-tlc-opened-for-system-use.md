# SUMMARY: Microsoft, TLCs opened by Windows for system use

## Claim
Windows itself opens certain TLCs:
- Mouse (0x01/0x01-0x02) and keyboard (0x01/0x06-0x07): exclusively.
- Consumer control (0x0C/0x01) and system control (0x01/0x80): shared.
- Vendor-defined pages (0xFF00 and up) are not on the list.

## Method
Official reference table.

## Result
- HYPACE's mouse TLC (0x01/0x02) and boot-keyboard TLC (0x01/0x06) cannot be opened for read/write by an application.
- The vendor TLCs (0xFF02..0xFF06) are not opened by the system, so an application can open them read/write.
- Consumer and system control TLCs inside the vendor interface are shared.

## Evidence tier
1 (official documentation).

## Performance
Not applicable.

## Correctness
- Proven: the table.
- Corroborated: async-hid issue 35 shows "Access is denied (0x80070005)" when opening a keyboard TLC read/write (tier-2 maintainer explanation).
- hidapi-rs windows-native retries a failed open with no read/write access (source comment: "System devices, such as keyboards and mice, cannot be opened in read-write mode"). Such a handle opens, but WriteFile and ReadFile on it will not work. That consequence is inferred from the access mode and was not tested.

## Relevance to hyperpace
- Filter enumeration to the vendor usage page. Opening the first 0x3554:0xFB16 path returned can land on an exclusive mouse or keyboard TLC.
- With hidapi-rs windows-native, that wrong open may even succeed with no access, and the failure only appears later at write time.
- Error messages should name the collection that was attempted.
