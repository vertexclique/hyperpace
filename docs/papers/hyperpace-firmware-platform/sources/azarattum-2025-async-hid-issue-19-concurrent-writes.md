URL: https://github.com/sidit77/async-hid/issues/19
Retrieved: 2026-09-15
Source type: GitHub issue with maintainer (sidit77) comments; OPEN at retrieval.
Retrieval method: `gh issue view 19 -R sidit77/async-hid --json ...` (verbatim, truncated).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`Concurrent writes on windows | Azarattum | 2025-03-24T14:11:14Z | OPEN`

> I have multiple concurrent writes to HID in my program. Everything works fine on MacOS, but on Windows it results in `Another write operation is in progress` error.

sidit77, 2025-03-25:
> This behavior is an intentional stopgap solution and should be "fixed" in #16 where I removed the internal mutex and changed the function signature from `&self`to `&mut self`.
>
> If you really need to write from multiple sources you should be able to wrap the writer into an async mutex yourself.

Azarattum, 2025-03-25:
> Also there is another small difference that I've noticed with Windows. It seems like it always requires you to send report ID as the first byte.

sidit77, 2025-03-26:
> So, I've looked a bit more into the report id issue and I believe that the first byte should be the report id on all platforms:
> ...
> **That being said**, looking at this right now, using the first byte as report id seems like a pretty bad API that this library inherited from its predecessors. I think having something like `report_id: Option<NonZeroU8>` would be a lot clearer.
