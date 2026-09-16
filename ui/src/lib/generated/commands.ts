// Companion to crates/hyperpace-app/src/command_list.rs. Kept in sync by hand; the Rust test
// `frontend_command_list_matches_the_registered_handlers` in
// crates/hyperpace-app/tests/frontend_contract.rs parses this file and asserts it equals
// `command_list::COMMANDS` exactly, so a command renamed, added or removed on the Rust side
// without a matching edit here fails the Rust test suite instead of failing silently at runtime.
//
// `tauri.ts`'s `invoke` only accepts a `CommandName`, so a call site that misspells or drops a
// command name fails to type-check as well.

export const COMMAND_NAMES = [
	'list_devices',
	'connect',
	'disconnect',
	'device_state',
	'read_settings',
	'write_setting',
	'set_button',
	'get_button_keystroke',
	'save_macro',
	'list_macros',
	'delete_macro',
	'set_profile',
	'factory_reset',
	'pair_receiver',
	'receiver_light',
	'read_receiver_light',
	'export_config',
	'import_config',
	'firmware_list',
	'firmware_import',
	'firmware_install',
	'firmware_check_for_updates',
	'firmware_watch_check',
	'app_settings'
] as const;

export type CommandName = (typeof COMMAND_NAMES)[number];
