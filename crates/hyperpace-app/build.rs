//! Tauri build script: embeds `tauri.conf.json` and autogenerates the ACL permission for every
//! command in `commands::COMMANDS`, so `capabilities/default.json` only has to name them.

#[path = "src/command_list.rs"]
mod command_list;

fn main() {
    let result = tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(command_list::COMMANDS)),
    );
    if let Err(error) = result {
        eprintln!("tauri-build failed: {error}");
        std::process::exit(1);
    }
}
