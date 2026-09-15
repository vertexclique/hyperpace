# SUMMARY: charlesschaefer-2025-flatpak-tray-icon-temp-dir

## Claim
Under Flatpak, the tray icon shows as "unknown". The AppIndicator backend writes the icon PNG to a runtime directory that the tray host cannot read from inside the sandbox.

## Method
User debugging, documented in a tauri-docs PR. Corroborated by the tray-icon gtk backend source, which writes `tray-icon-{id}-{counter}.png` on each icon change.

## Result
Workaround: grant access, or change the temp dir path (`temp_dir_path`).

## Evidence tier
3, corroborated by tier 1 source.

## Performance
Confirms file I/O on every icon update on the AppIndicator backend.

## Correctness
Specific to Flatpak. Per the reporter, AppImage, deb and rpm are unaffected.

## Relevance to hyperpace
Matters only if Flatpak is ever a target. It reinforces throttling icon updates on Linux with Tauri 2.
