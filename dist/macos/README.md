# macOS bundle

No custom `Info.plist` fragment or entitlements file lives here, deliberately:

- **Category and copyright.** Tauri already writes `LSApplicationCategoryType`
  from the shared `bundle.category` field (see `../README.md`), and
  `NSHumanReadableCopyright` from `bundle.copyright`. A merged `Info.plist`
  would only repeat those.
- **No sandbox entitlements needed.** Hyperpace is distributed outside the
  Mac App Store and is not sandboxed, so IOKit HID access needs no
  `com.apple.security.device.usb` entitlement; that entitlement only matters
  under App Sandbox, which is out of scope (plan: unsigned builds for now).
  Hardened runtime (Tauri's own default once a `signingIdentity` is set)
  restricts code injection and unsigned executable memory, not device
  access, and this app needs no exception to it.
- **Input Monitoring is a runtime TCC prompt, not a plist key.** The vendor
  HID usage page this device uses (0xFF02) is not the keyboard input page
  TCC gates, and even where it were, macOS grants that permission through a
  system dialog with no `Info.plist` usage-description key to set. The plan
  already lists this as needing confirmation on real Mac hardware
  (`docs/plans/hyperpace.md`, deferred items).

If code signing and notarization are picked up later (currently deferred by
operator choice), an entitlements file becomes relevant only if the app
later needs a sandbox exception it does not need today; add it then, driven
by whatever Apple actually requires for the tools in use at that time.

DMG background art and icon placement (`mac.dmg`) are skipped for the same
reason as the AppImage and Windows installer graphics: no design assets
exist in this repository to place there.
