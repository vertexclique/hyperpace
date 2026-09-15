URL: https://learn.microsoft.com/en-us/windows/win32/secbp/understanding-pe-signatures
Retrieved: 2026-09-15 (WebFetch; page metadata `ms.date: 2025-07-24`, `updated_at: 2025-07-31T05:09:00Z`, git_commit_id 4af4ff75fe3bdb0e631c2ad6e87b8b2a85dfcc22)
Source type: Microsoft official documentation (primary)

Title: "Understanding Executable File Signing - Win32 apps"

## Verbatim

> In Windows, a file's digital signature may be stored in either a catalog signature, an embedded signature, or both. Catalog Signatures are files that contain signatures for one or more other files, while embedded signatures are stored in file-format-specific structures that allow embedding a signature within the bytes of the file itself.

> In the case of file types for which no SIP is installed, the entire content of the file may be hashed (as in the case of a text file) and stored in a Catalog file.

> In a Windows PE file, digital signatures can be "embedded" in a location specified by the Certificate Table entry in Optional Header Data Directories. When Authenticode is used to sign a Windows PE file, the algorithm that calculates the file's hash value excludes certain PE fields.

> The Software Publisher Trust Provider SIP for PE files hashes the executable content of the file, but necessarily omits the file's checksum (which changes when a signature is embedded) and the Certificate Table directory (which is populated by the file's signatures during signing).
>
> The fact that the Software Publisher Trust Provider SIP does not hash all of the bytes of the file (i.e. it's not a "flat file hash") when calculating the signature means that it is possible to embed additional content into a signed PE file without breaking its signature. ... While the injected content is not directly executable, a vulnerable application might incorrectly expect it to be trustworthy (because the rest of the file is correctly signed) and act upon it.

> When the EnableCertPaddingCheck DWORD is present and set to 1, [WinVerifyTrust] will validate that all PKCS#7 padding bytes in the WIN_CERTIFICATE structure are set to 0.

(Typographic apostrophes and quotes in the original are rendered as ASCII here.)
