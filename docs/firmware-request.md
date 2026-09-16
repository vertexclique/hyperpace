# Firmware request: what to send the vendor

Route 2 of the three firmware acquisition routes named in `docs/plans/hyperpace.md`'s firmware
paragraph. No genuine HYPACE firmware image was found on any channel reached during research
(`docs/papers/hyperpace-firmware-platform/FIRMWARE-VERDICT.md`); asking the vendor directly is the
only route that can plausibly return a complete version history, which is what rollback needs.

Paste the message below into Lofree's support channel (their site lists a contact/support form and
a Discord). Nothing here needs to change before sending it; the hardware identifiers are drawn
directly from this mouse's own USB descriptors and the vendor's own published configuration, not
guessed.

## Message

> Subject: Firmware update packages needed for HYPACE mouse and receiver (all versions)
>
> Hello,
>
> I own a Lofree HYPACE mouse and would like to request the official firmware update packages for
> it and its 2.4 GHz receiver, including every version you have available, not only the current
> one.
>
> My hardware identifies itself as:
>
> - Mouse, wired connection: USB vendor id 0x3554, product id 0xFB14 (reported device version
>   0x0300)
> - Receiver, 2.4 GHz dongle: USB vendor id 0x3554, product id 0xFB16 (reported device version
>   0x0216)
> - Device model id reported by your own driver: cid 102
>
> Could you send:
>
> 1. Every firmware package you have for the mouse, every version, not only the latest.
> 2. Every firmware package you have for the receiver, every version, for whichever receiver
>    variant (1K, 2K, 4K or 8K) matches product id 0xFB16 on my unit. Could you also confirm which
>    variant that is? I was not able to determine it from your public configuration files alone.
> 3. Please send the complete, vendor-built update package exactly as your own updater tool would
>    use it (the full file, with its header and update commands intact), not a raw firmware payload
>    and not a rebuilt or re-packaged image. A raw payload or a rebuilt image cannot be flashed: the
>    update procedure needs the whole package your build process produces, not just the firmware
>    bytes inside it.
>
> Having more than one version lets me roll back if a newer release causes a problem, which is why
> I am asking for the full version history rather than only the newest release.
>
> Thank you for your help.

## Why the wording is exact

- **USB ids and device versions** come from this mouse's own passive USB descriptor read
  (`firmware/archive/SOURCES.md`), not a guess: `Compx Hypace@Lofree`, wired PID `0xFB14` at device
  version `0x0300`, receiver PID `0xFB16` at device version `0x0216`.
- **cid 102** is the model id the vendor's own production `/mouse` driver config uses for this
  product (`docs/papers/hyperpace-firmware-platform/sources/local-2026-vendor-page-deployments.md`);
  a newer chooser deployment separately uses cid 62 for the same page, so the message does not
  claim to know which one the device itself reports over the wire, only which the vendor's own
  driver assigns.
- **"Complete, vendor-built package, not a raw payload"** matters because nothing in any reachable
  implementation can synthesize a package from a raw firmware dump: the reset, prepare and data
  commands are copied verbatim from a vendor-built header, and this mouse's own MCU family's
  integrity check value matches no tested algorithm
  (`docs/papers/hyperpace-firmware-platform/FIRMWARE-VERDICT.md` section 1). A raw payload is
  unflashable by construction, so asking for anything less than the full package would not help.
- **"Every version"** is what rollback needs. A single version, even a genuine one, cannot be
  rolled back from; the archive needs at least two versions per target before rollback becomes
  possible at all.

## After a reply arrives

Save whatever arrives without opening or running it, then use Hyperpace's own import path (the
Firmware screen's "Import a package" action, or `firmware_import`): it parses the file, checks its
identity markers against this hardware, and refuses anything that does not match, before it is ever
offered for install.
