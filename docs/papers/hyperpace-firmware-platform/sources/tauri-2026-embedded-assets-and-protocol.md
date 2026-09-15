URL: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-codegen/src/embedded_assets.rs
Additional URLs: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/manager/mod.rs ; https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/Cargo.toml ; https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/CHANGELOG.md ; https://v2.tauri.app/security/csp/ ; https://v2.tauri.app/concept/size/
Retrieved: 2026-09-15 (tauri `dev` commit bca4ca58da02f182ef00ef1165e40e400a3cd8dd; tauri-docs `v2` commit a6b59b7819f34edfacf365ce15fcd53c9f00ba11)
Source type: framework source code and official documentation (primary)

## Verbatim, tauri-codegen embedded_assets.rs: what embedding means

```rust
/// Represent a directory of assets that are compressed and embedded.
///
/// This is the compile time generation of [`tauri_utils::assets::Assets`] from a directory. Assets
/// from the directory are added as compiler dependencies by dummy including the original,
/// uncompressed assets.
///
/// The assets are compressed during this runtime, and can only be represented as a [`TokenStream`]
/// through [`ToTokens`]. The generated code is meant to be injected into an application to include
/// the compressed assets in that application's binary.
#[derive(Default)]
pub struct EmbeddedAssets {
  assets: BTreeMap<AssetKey, (PathBuf, PathBuf)>,
  csp_hashes: CspHashes,
```

## Verbatim, tauri-utils assets.rs: runtime lookup decompresses the whole asset into a new Vec per call

```rust
  /// Get an asset by key.
  #[cfg(feature = "compression")]
  pub fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    let &(mut asdf) = self.assets.get(key.as_ref())?;
    // with the exception of extremely small files, output should usually be
    // at least as large as the compressed version.
    let mut buf = Vec::with_capacity(asdf.len());
    brotli::BrotliDecompress(&mut asdf, &mut buf).ok()?;
    Some(Cow::Owned(buf))
  }
```

## Verbatim, compression level (embedded_assets.rs)

```rust
  /// Use highest compression level for release, the fastest one for everything else
  #[cfg(feature = "compression")]
  fn compression_settings() -> BrotliEncoderParams {
    let mut settings = BrotliEncoderParams::default();

    // the following compression levels are hand-picked and are not min-maxed.
    // they have a good balance of runtime vs size for the respective profile goals.
    // see the "brotli" section of this comment https://github.com/tauri-apps/tauri/issues/3571#issuecomment-1054847558
    if cfg!(debug_assertions) {
      settings.quality = 2
    } else {
      settings.quality = 9
    }

    settings
  }
```

## Verbatim, crates/tauri/Cargo.toml features (Tauri 2.11.5 tree)

```
default = [
  "wry",
  "compression",
  "common-controls-v6",
  "dynamic-acl",
  "x11",
```
```
compression = ["tauri-macros/compression", "tauri-utils/compression"]
wry = ["webview2-com", "webkit2gtk", "tauri-runtime-wry"]
```
tauri-codegen/Cargo.toml: `compression = ["brotli", "tauri-utils/compression"]`

## Verbatim, crates/tauri/src/manager/mod.rs: the custom protocol URL per platform

```rust
  /// The `tauri` custom protocol URL we use to serve the embedded assets.
  /// Returns `tauri://localhost` or its `wry` workaround URL `http://tauri.localhost`/`https://tauri.localhost`
  pub(crate) fn tauri_protocol_url(&self, https: bool) -> Cow<'_, Url> {
    if cfg!(windows) || cfg!(target_os = "android") {
      let scheme = if https { "https" } else { "http" };
      Cow::Owned(Url::parse(&format!("{scheme}://tauri.localhost")).unwrap())
    } else {
      Cow::Owned(Url::parse("tauri://localhost").unwrap())
    }
  }
```

## Verbatim, CHANGELOG tauri 2.11.3 (Performance Improvements)

> [`e6083a111`] ([#15220]) Load `tauri://` custom protocol handlers asynchronously to speed up load time

## Verbatim, CSP docs (https://v2.tauri.app/security/csp/)

> Local scripts are hashed, styles and external scripts are referenced using a cryptographic nonce,
> which prevents unallowed content from being loaded.

> The CSP protection is only enabled if set on the Tauri configuration file.
> You should make it as restricted as possible, only allowing the webview to load assets
> from hosts you trust, and preferably own.
> At compile time, Tauri appends its nonces and hashes to the relevant CSP attributes automatically
> to bundled code and assets, so you only need to worry about what is unique to your application.

```json title="tauri/examples/api/src-tauri/tauri.conf.json"
  "csp": {
        "default-src": "'self' customprotocol: asset:",
        "connect-src": "ipc: http://ipc.localhost",
        "font-src": ["https://fonts.gstatic.com"],
        "img-src": "'self' asset: http://asset.localhost blob: data:",
        "style-src": "'unsafe-inline' 'self' https://fonts.googleapis.com"
      },
```

## Verbatim, App Size docs (https://v2.tauri.app/concept/size/)

```toml
# src-tauri/Cargo.toml
[profile.dev]
incremental = true # Compile your binary in smaller steps.

[profile.release]
codegen-units = 1 # Allows LLVM to perform better optimization.
lto = true # Enables link-time-optimizations.
opt-level = "s" # Prioritizes small binary size. Use `3` if you prefer speed.
panic = "abort" # Higher performance by disabling panic handlers.
strip = true # Ensures debug symbols are removed.
```

```json title=tauri.conf.json
{
  "build": {
    "removeUnusedCommands": true
  }
}
```

> to remove commands that're never allowed in your capability files (ACL), so you don't have to pay for what you don't use

> This feature requires `tauri@2.4`, `tauri-build@2.1`, `tauri-plugin@2.1` and `tauri-cli@2.4`
