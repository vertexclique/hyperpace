//! The canonical list of every Tauri command name this crate registers.
//!
//! Read by `build.rs` (via `#[path]`, since a build script is its own crate) to autogenerate each
//! command's `allow-*`/`deny-*` ACL permission, which `capabilities/default.json` then names.
//! `lib.rs`'s `tauri::generate_handler!` call needs the same set as literal function paths, a
//! syntactic form this list cannot produce for it, so the two are kept in sync by hand; a command
//! added to one and not the other fails loudly (an unregistered handler answers "command not
//! found", and an unlisted command has no capability to allow it), never silently.

/// Every Tauri command name in `docs/architecture/api-contract.md`'s command list, exactly as
/// registered with `tauri::generate_handler!` in `lib.rs`.
pub const COMMANDS: &[&str] = &[
    "list_devices",
    "connect",
    "disconnect",
    "device_state",
    "read_settings",
    "write_setting",
    "set_button",
    "get_button_keystroke",
    "save_macro",
    "list_macros",
    "delete_macro",
    "set_profile",
    "factory_reset",
    "pair_receiver",
    "receiver_light",
    "read_receiver_light",
    "export_config",
    "export_config_file",
    "import_config_file",
    "import_config",
    "firmware_list",
    "firmware_import",
    "firmware_install",
    "firmware_check_for_updates",
    "firmware_watch_check",
    "app_settings",
    "data_overview",
    "list_events",
    "clear_events",
    "list_profile_snapshots",
    "save_profile_snapshot",
    "restore_profile_snapshot",
    "rename_profile_snapshot",
    "delete_profile_snapshot",
    "firmware_delete",
    "reset_app_setting",
    "export_store",
    "import_store",
];
