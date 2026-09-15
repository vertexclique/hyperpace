URL: https://v2.tauri.app/distribute/windows-installer/
Additional URLs: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/distribute/windows-installer.mdx (commit a6b59b78) ; https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle.rs (commit bca4ca58)
Retrieved: 2026-09-15
Source type: official documentation and bundler source (primary)

Note: a warning emoji in the original `skip` table row is omitted here.

## Verbatim, docs

> Tauri applications for Windows are either distributed as Microsoft Installers (`.msi` files) using the [WiX Toolset v3]
> or as setup executables (`-setup.exe` files) using [NSIS].
>
> Please note that `.msi` installers can **only be created on Windows** as WiX can only run on Windows systems.
> Cross-compilation for NSIS installers is shown below.

> Cross compiling Windows apps on Linux and macOS hosts is possible with caveats when using [NSIS].
> It is not as straight forward as compiling on Windows directly and is not tested as much.
> Therefore it should only be used as a last resort if local VMs or CI solutions like GitHub Actions don't work for you.

> Signing cross compiled Windows installers requires an external signing tool.

> Since Tauri officially only supports the MSVC Windows target, the setup is a bit more involved.

> But on many other distributions you have to compile NSIS yourself or download Stubs and Plugins manually that weren't included in the distro's binary package.

```sh
cargo install --locked cargo-xwin
```
```
cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
```
> The build output will then be in `target/x86_64-pc-windows-msvc/release/bundle/nsis/`.

## Verbatim, WebView2 installation options table

| Installation Method | Requires Internet Connection? | Additional Installer Size | Notes |
| :-- | :-- | :-- | :-- |
| `downloadBootstrapper` | Yes | 0MB | `Default` Results in a smaller installer size, but is not recommended for Windows 7 deployment via `.msi` files. |
| `embedBootstrapper` | Yes | ~1.8MB | Better support on Windows 7 for `.msi` installers. |
| `offlineInstaller` | No | ~127MB | Embeds WebView2 installer. Recommended for offline environments. |
| `fixedVersion` | No | ~180MB | Embeds a fixed WebView2 version. |
| `skip` | No | 0MB | Not recommended. Does not install the WebView2 as part of the Windows Installer. |

> On Windows 10 (April 2018 release or later) and Windows 11, the WebView2 runtime is distributed as part of the operating system.

> Using the runtime provided by the system is great for security as the webview vulnerability patches are managed by Windows.

> Your application WILL NOT work if the user does not have the runtime installed and won't attempt to install it.

> If your app requires features only available in newer Webview2 versions (such as custom URI schemes), you can instruct the Windows installer
> to verify the current Webview2 version and run the Webview2 bootstrapper if it does not match the target version.

## Verbatim, bundler source crates/tauri-bundler/src/bundle.rs: macOS bundle types compile only on a macOS host

```rust
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
```
```rust
  #[cfg(target_os = "macos")]
  let bundle_type = match package_type {
    // NSIS installers can be built in macOS using cargo-xwin
```
```rust
    let bundle_paths = match package_type {
      #[cfg(target_os = "macos")]
      PackageType::MacOsBundle => macos::app::bundle_project(settings)?,
      #[cfg(target_os = "macos")]
      PackageType::IosBundle => macos::ios::bundle_project(settings)?,
      // dmg is dependent of MacOsBundle, we send our bundles to prevent rebuilding
      #[cfg(target_os = "macos")]
      PackageType::Dmg => {
```
