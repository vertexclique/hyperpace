URL: https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/updater/src/updater.rs
Additional URLs: https://github.com/tauri-apps/plugins-workspace/issues/2672 ; https://github.com/linuxdeploy/linuxdeploy-plugin-appimage (README)
Retrieved: 2026-09-15 (plugins-workspace `v2` commit 0850317b, updater 2.11.0)
Source type: plugin source code (primary); GitHub feature request by a Tauri maintainer (tier 2)

## Verbatim, `Update::download` (updater.rs lines ~680-742): whole artifact buffered in memory, then signature-verified

```rust
    pub async fn download<C: FnMut(usize, Option<u64>), D: FnOnce()>(
        &self,
        mut on_chunk: C,
        on_download_finish: D,
    ) -> Result<Vec<u8>> {
```
```rust
        let mut buffer = Vec::new();

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            on_chunk(chunk.len(), content_length);
            buffer.extend(chunk);
        }
        on_download_finish();

        verify_signature(&buffer, &self.signature, &self.context.config.pubkey)?;

        Ok(buffer)
    }
```

## Verbatim, signature verification (updater.rs lines ~1523-1534)

```rust
// Validate signature
fn verify_signature(data: &[u8], release_signature: &str, pub_key: &str) -> Result<()> {
    // we need to convert the pub key
    let pub_key_decoded = base64_to_string(pub_key)?;
    let public_key = PublicKey::decode(&pub_key_decoded)?;
    let signature_base64_decoded = base64_to_string(release_signature)?;
    let signature = Signature::decode(&signature_base64_decoded)?;

    // Validate signature or bail out
    public_key.verify(data, &signature, true)?;
    Ok(())
}
```

## Verbatim, issue #2672 (OPEN, 2025-05-01) "Built-in support for appimage zsync delta updates"

FabianLars (opening post):
> In https://github.com/tauri-apps/tauri/pull/12491 i'll add support for zsync updates. For now this is only relevant for external appimage updaters but i think it makes sense to make our updater to support it for delta self updates as well.

Samueru-sama (2025-05-01):
> Note that the name of the AppImage or `.zsync` file **cannot** be changed after these steps are done

## Search listing for delta updates (2026-09-15)

```
gh search issues --repo tauri-apps/plugins-workspace "delta update": 2672 open 2025-05-01 Built-in support for appimage zsync delta updates
gh search issues --repo tauri-apps/tauri "delta updates": (no results)
gh search issues --repo tauri-apps/tauri "differential update": (no results)
```

## Verbatim, linuxdeploy-plugin-appimage README (update information and compression knobs)

> - `LDAI_UPDATE_INFORMATION="..."`: embed [update information](https://github.com/AppImage/AppImageSpec/blob/master/draft.md#update-information) in the AppImage, and generate corresponding `.zsync` file

> - `LDAI_COMP=...`: compression algorithm appimagetool/mksquashfs should use (e.g., `xz`, `gzip`), see [appimagetool's repository](https://github.com/AppImage/AppImageKit/) for more information
