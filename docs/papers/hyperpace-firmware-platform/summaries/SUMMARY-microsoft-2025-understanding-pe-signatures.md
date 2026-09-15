# SUMMARY: microsoft-2025-understanding-pe-signatures

## Claim
- Authenticode for PE files hashes the executable content but omits the checksum and the Certificate Table.
- So extra unsigned content can be injected into padding or unauthenticated attributes without breaking the signature; `EnableCertPaddingCheck` mitigates this.
- Files without a SIP are hashed whole and signed via catalogs.

## Method
Microsoft Learn documentation (updated 2025-07-31).

## Result
Data compiled into PE sections, as `include_bytes!` produces, is inside the Authenticode hash. Files installed beside the exe are not covered by the exe's signature.

## Evidence tier
1.

## Performance
Not applicable.

## Correctness
That section data is hashed follows from "hashes the executable content of the file" together with the exclusion list. The exact algorithm document (the Authenticode PE format paper) was not fetched. Whether Windows re-verifies Authenticode on each launch of an installed exe was not verified.

## Relevance to hyperpace
On Windows, `include_bytes!` firmware inherits the exe signature; NSIS-installed resource files do not. In both cases, what the flasher can rely on at the moment of flashing is an app-level check: a SHA-256 manifest compiled into the signed binary.
