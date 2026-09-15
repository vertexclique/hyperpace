# SUMMARY: tauri-2026-updater-plugin

## Claim
tauri-plugin-updater 2.11.0 requires minisign signatures ("This cannot be disabled").
- Keys come from `tauri signer generate`. The private key must be in env vars at build time.
- Losing the private key ends updates for installed users.
- Updatable artifacts: AppImage (Linux), `.app.tar.gz` (macOS), NSIS `-setup.exe` and MSI (Windows).
- Endpoints can be a static JSON (tauri-action generates `latest.json`) or a dynamic server that returns 204 for no update.
- TLS is enforced in production.

## Method
The docs and `updater.rs`.

## Result
- Source also installs `.deb` via `dpkg -i`, and `.rpm`. To escalate it tries pkexec, then zenity or kdialog, then terminal sudo.
- The bundle type comes from a value the bundler patches into the binary. An unknown type falls back to the AppImage path.
- A pending `--no-binary-patching` flag skips that patch.

## Evidence tier
1.

## Performance
See tauri-2026-updater-download-and-delta: the full artifact is buffered in RAM and there are no deltas.

## Correctness
- The docs table lists only AppImage for Linux, while the source supports deb and rpm: the docs lag the source.
- AppImage self-update renames the running file and writes the new one, so the AppImage location must be writable (inferred from code).

## Relevance to hyperpace
Feasible for AppImage, NSIS and macOS. Key custody and CI secrets are operator decisions. With firmware embedded, every app update re-downloads all firmware bytes.
