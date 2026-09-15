URL: https://v2.tauri.app/develop/resources/
Additional URLs: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-utils/src/platform.rs ; https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/linux/debian.rs ; https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/macos/app.rs ; https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi ; https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-utils/src/assets.rs
Retrieved: 2026-09-15 (tauri `dev` commit bca4ca58; tauri-docs `v2` commit a6b59b78)
Source type: official documentation and framework/bundler source (primary)

## Verbatim, docs "Embedding Additional Files"

> You may need to include additional files in your application bundle that aren't part of your frontend (your `frontendDist`) directly or which are too big to be inlined into the binary. We call these files `resources`.

```json title=tauri.conf.json
{
  "bundle": {
    "resources": [
      "./path/to/some-file.txt",
      "/absolute/path/to/textfile.txt",
      "../relative/path/to/jsonfile.json",
      "some-folder/",
      "resources/**/*.md"
    ]
  }
}
```

> The bundled files will be in `$RESOURCES/` with the original directory structure preserved,
> for example: `./path/to/some-file.txt` -> `$RESOURCE/path/to/some-file.txt`

> To learn about where `$RESOURCE` resolves to on each platforms, see the documentation of [`resource_dir`]

```rust
tauri::Builder::default()
  .setup(|app| {
    let resource_path = app.path().resolve("lang/de.json", BaseDirectory::Resource)?;
```

## Verbatim, tauri-utils platform.rs `resource_dir` doc comment

```rust
/// Computes the resource directory of the current environment.
///
/// ## Platform-specific
///
/// - **Windows:** Resolves to the directory that contains the main executable.
/// - **Linux:** When running in an AppImage, the `APPDIR` variable will be set to
///   the mounted location of the app, and the resource dir will be `${APPDIR}/usr/lib/${exe_name}`.
///   If not running in an AppImage, the path is `/usr/lib/${exe_name}`.
///   When running the app from `src-tauri/target/(debug|release)/`, the path is `${exe_dir}/../lib/${exe_name}`.
/// - **macOS:** Resolves to `${exe_dir}/../Resources` (inside .app).
```

## Verbatim, platform.rs `resource_dir_from` body (Windows, Linux, macOS)

```rust
  if cfg!(target_os = "windows")
    || ((len >= 2 && parts[len - 2] == "target") || (len >= 3 && parts[len - 3] == "target"))
      && is_cargo_output_directory(exe_dir)
  {
    return Ok(exe_dir.to_path_buf());
  }
```
```rust
  #[cfg(target_os = "linux")]
  {
    // (canonicalize checks for existence, so there's no need for an extra check)
    res = if let Ok(bundle_dir) = exe_dir
      .join(format!("../lib/{}", package_info.name))
      .canonicalize()
    {
      Ok(bundle_dir)
    } else if let Some(appdir) = &env.appdir {
      let appdir: &std::path::Path = appdir.as_ref();
      Ok(PathBuf::from(format!(
        "{}/usr/lib/{}",
        appdir.display(),
        package_info.name
      )))
    } else {
      // running bundle
      Ok(PathBuf::from(format!("/usr/lib/{}", package_info.name)))
    };
  }

  #[cfg(target_os = "macos")]
  {
    res = exe_dir
      .join("../Resources")
      .canonicalize()
      .map_err(Into::into);
  }
```
```rust
// Variable holding the type of bundle the executable is stored in. This is modified by binary
// patching during build
#[used]
// Marked as `mut` because it could get optimized away without it,
// see https://github.com/tauri-apps/tauri/pull/13812
static mut __TAURI_BUNDLE_TYPE: &str = "__TAURI_BUNDLE_TYPE_VAR_UNK";
```

## Verbatim, bundler placement

debian.rs:
```
//         usr/lib/foobar/...                        # Other resource files
//
// For cargo-bundle, we put bundle resource files under /usr/lib/package_name/,
```
```rust
  let resource_dir = data_dir.join("usr/lib").join(settings.product_name());
  settings.copy_resources(&resource_dir)
```

macos/app.rs:
```
//         Resources      # Data files such as images, sounds, translations and nib files
75:  let resources_dir = bundle_directory.join("Resources");
97:  settings.copy_resources(&resources_dir)?;
253:  plist.insert("CSResourcesFileMapped".into(), true.into());
```

windows/nsis/installer.nsi:
```
  ; Copy main executable
  File "${MAINBINARYSRCPATH}"

  ; Copy resources
  {{#each resources_dirs}}
    CreateDirectory "$INSTDIR\\{{this}}"
  {{/each}}
  {{#each resources}}
    File /a "/oname={{this.[1]}}" "{{no-escape @key}}"
  {{/each}}
```

## Verbatim, v3 alpha change to unbundled resource resolution (tauri-v3.0.0-alpha.0 release notes)

> `tauri-build` no longer copies the configured resources to the cargo target directory; on desktop, unbundled apps (`tauri dev` / `cargo run`) now resolve resources at runtime from their source paths instead.

## Verbatim, tauri-utils assets.rs: embedded frontend assets are decompressed per request into a new buffer

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

## Search for built-in integrity features (grep over tauri-docs develop/, security/, distribute/, plugin/ for checksum|sha256|integrity, 2026-09-15)

Matches were only: CSP `sha256-` hashes (security/http-headers.mdx), Flatpak/AUR manifest checksums (distribute/flatpak.mdx, distribute/aur.mdx), Windows signing `digestAlgorithm` (distribute/Sign/windows.mdx), SQL migration wording, and deep-link `sha256_cert_fingerprints`. No page describes runtime verification of bundled resources or embedded blobs. The only built-in signature check found in source is the updater's minisign verification of update artifacts (see tauri-2026-updater-plugin.md).
