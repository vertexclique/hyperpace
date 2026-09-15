URL: https://v2.tauri.app/plugin/updater/
Additional URLs: https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/updater/src/updater.rs ; https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/updater/Cargo.toml ; https://github.com/tauri-apps/tauri/blob/dev/.changes/disable-binary-patching.md
Retrieved: 2026-09-15 (plugins-workspace `v2` commit 0850317b, plugin version 2.11.0; tauri `dev` commit bca4ca58)
Source type: official plugin documentation and source (primary)

## Verbatim, docs: signing is mandatory

> Tauri's updater needs a signature to verify that the update is from a trusted source. This cannot be disabled.

> 2. The private key, which is used to sign your installer files. You should NEVER share this key with anyone. Also, if you lose this key you will NOT be able to publish new updates to the users that have the app already installed. It is important to store this key in a safe place!

```
cargo tauri signer generate -w ~/.tauri/myapp.key
```
```sh
export TAURI_SIGNING_PRIVATE_KEY="Path or content of your private key"
# optionally also add a password
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
```
> While building your update artifacts, you need to have the private key you generated above in your environment variables. `.env` files do _not_ work!

## Verbatim, docs: artifacts per OS (createUpdaterArtifacts: true)

> On Linux, Tauri will create the normal AppImage inside the `target/release/bundle/appimage/` folder:
>
> - `myapp.AppImage` - The standard app bundle. It will be re-used by the updater.
> - `myapp.AppImage.sig` - The signature of the updater bundle.
>
> On macOS, Tauri will create a .tar.gz archive from the application bundle inside the target/release/bundle/macos/ folder:
>
> - `myapp.app` - The standard app bundle.
> - `myapp.app.tar.gz` - The updater bundle.
> - `myapp.app.tar.gz.sig` - The signature of the update bundle.
>
> On Windows, Tauri will create the normal MSI and NSIS installers inside the target/release/bundle/msi/ and target/release/bundle/nsis folders:
>
> - `myapp-setup.exe` - The standard app bundle. It will be re-used by the updater.
> - `myapp-setup.exe.sig` - The signature of the update bundle.
> - `myapp.msi` - The standard app bundle. It will be re-used by the updater.
> - `myapp.msi.sig` - The signature of the update bundle.

## Verbatim, docs: configuration

> `pubkey` | This has to be the public key generated from the Tauri CLI in the step above. It **cannot** be a file path!
> `endpoints` | This must be an array of endpoint URLs as strings. TLS is enforced in production mode. Tauri will only continue to the next url if a non-2XX status code is returned!

> - `"passive"`: There will be a small window with a progress bar. The update will be installed without requiring any user interaction. Generally recommended and the default mode.

> Note that Tauri will validate the whole file before checking the version field, so make sure all existing platform configurations are valid and complete.

> Your server should respond with a status code of [`204 No Content`] if there is no update available.

## Verbatim, source updater.rs: Linux install paths

```rust
    fn install_inner(&self, bytes: &[u8]) -> Result<()> {
        match installer_for_bundle_type(bundle_type()) {
            Some(Installer::Deb) => self.install_deb(bytes),
            Some(Installer::Rpm) => self.install_rpm(bytes),
            _ => self.install_appimage(bytes),
        }
    }
```
```rust
                    // create a backup of our current app image
                    std::fs::rename(&self.extract_path, tmp_app_image)?;
```
```
1127:        self.try_tmp_locations(bytes, "dpkg", "-i", "deb")
1183:        // 1. First try using pkexec (graphical sudo prompt)
1196:        // 2. Try zenity or kdialog for a graphical sudo experience
1204:        // 3. Final fallback: terminal sudo
```
```
1023:/// Linux (AppImage, Deb, RPM)
459:// replace {{current_version}}, {{target}}, {{arch}} and {{bundle_type}} in t...
```
(line 459 was truncated by the capture tool at the ellipsis)

## Verbatim, signature verification dependency

```
plugins/updater/Cargo.toml:50:minisign-verify = "0.2"
plugins/updater/src/updater.rs:21:use minisign_verify::{PublicKey, Signature};
```

## Verbatim, tauri `dev` pending change `.changes/disable-binary-patching.md`

> Add a `--no-binary-patching` flag to `tauri build` and `tauri bundle`. When set, the bundler skips patching the main executable with bundle type information (and the subsequent re-signing), leaving an already-signed binary untouched. Patching is only required when shipping multiple bundle types per platform that should each update with their own installer format.
