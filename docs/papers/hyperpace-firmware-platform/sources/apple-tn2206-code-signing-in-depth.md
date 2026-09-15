URL: https://developer.apple.com/library/archive/technotes/tn2206/_index.html
Retrieved: 2026-09-15 (WebFetch; archived Apple technical note, the tool returned the quoted passages below)
Source type: Apple official technical note (primary, archived documentation)

Title: "Technical Note TN2206: macOS Code Signing In Depth"

## Resource Rules

> Systems before OS X Mavericks 10.9 documented a signing feature `(--resource-rules)` to control which files in a bundle should be sealed by a code signature. This feature has been obsoleted for Mavericks. Code signatures made in Mavericks and later always seal all files in a bundle; there is no need to specify this explicitly any more.
>
> It is thus no longer possible to exclude parts of a bundle from the signature. Bundles should be treated as read-only once they have been signed.

## Nested Code

> Always put code and data into their proper places. This applies to all signed code and is enforced by the code signing machinery regardless of how the code is distributed.
>
> Store Python, Perl, shell, and other script files and other non-Mach-O executables in your app's `Contents/Resources` directory. While it's possible to sign such executables and store them in `Contents/MacOS`, this is not recommended.

## Signing Modifies the Executable

> Signing a program will modify its main executable file. There are some situations where this will cause you trouble:
>
> * If your program has a self-verification mode that detects a change, your code may refuse to run.
> * Appending data to a Mach-O executable is expressly prohibited. Signature verifications on such files will fail.
>
> The obvious solution to these problems is to not meddle with your signed program after you've signed it with `codesign`.

## Related (WebSearch summary of Apple "Code Signing Tasks", not fetched)

The search tool summarized https://developer-rno.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html as saying codesign records signatures of a bundle's non-code files in `Contents/_CodeSignature/CodeResources`. Not verified by fetching.
