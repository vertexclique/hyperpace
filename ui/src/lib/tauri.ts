// Thin bridge to the Tauri command surface fixed in
// docs/architecture/api-contract.md. Every command returns
// Result<T, String> on the Rust side; a rejected promise here carries that
// String verbatim, so callers can show the real failure instead of a guess.
//
// This file never fabricates a response. When the webview is not running
// inside the Hyperpace desktop shell (the plain `npm run dev`/`preview`
// preview, or a browser tab), every call fails honestly rather than
// returning invented device data.

import { invoke as tauriInvoke, Channel } from '@tauri-apps/api/core';
import type { CommandName } from './generated/commands';
import type { DeviceEvent, FirmwareProgress, PairState } from './types';

export { Channel };

export const runningInTauri =
	typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

const NOT_IN_SHELL = 'Not running inside the Hyperpace desktop app.';

/**
 * Calls a Tauri command, surfacing the real error message on failure.
 *
 * `command` is restricted to `CommandName` (generated/commands.ts, kept in sync with
 * `command_list::COMMANDS` by the Rust test in `tests/frontend_contract.rs`), so a renamed or
 * misspelled command name fails to type-check instead of failing at runtime.
 */
export async function invoke<T>(command: CommandName, args?: Record<string, unknown>): Promise<T> {
	if (!runningInTauri) {
		throw new Error(NOT_IN_SHELL);
	}
	return tauriInvoke<T>(command, args);
}

/**
 * Opens the per-window device event channel the contract describes ("Events to the UI go over
 * one Channel<DeviceEvent> per window, re-subscribed when a destroyed window is recreated"). The
 * channel is handed to the `device_state` command's `channel` argument, the command that both
 * registers it and returns the current connection snapshot; `connect` and `disconnect` do not
 * take a channel.
 */
export function openDeviceEventChannel(onEvent: (event: DeviceEvent) => void): Channel<DeviceEvent> {
	const channel = new Channel<DeviceEvent>();
	channel.onmessage = onEvent;
	return channel;
}

/** Opens a fresh progress channel for one `firmware_install` call. */
export function openFirmwareProgressChannel(
	onProgress: (progress: FirmwareProgress) => void
): Channel<FirmwareProgress> {
	const channel = new Channel<FirmwareProgress>();
	channel.onmessage = onProgress;
	return channel;
}

/** Opens a fresh channel for one `pair_receiver` call, streaming each `GetPairState` poll as it
 * happens instead of only the final phase. */
export function openPairStateChannel(onUpdate: (state: PairState) => void): Channel<PairState> {
	const channel = new Channel<PairState>();
	channel.onmessage = onUpdate;
	return channel;
}
