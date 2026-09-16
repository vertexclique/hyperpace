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
import type { DeviceEvent } from './types';

export const runningInTauri =
	typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

const NOT_IN_SHELL = 'Not running inside the Hyperpace desktop app.';

/** Calls a Tauri command, surfacing the real error message on failure. */
export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	if (!runningInTauri) {
		throw new Error(NOT_IN_SHELL);
	}
	return tauriInvoke<T>(command, args);
}

/**
 * Opens the per-window device event channel described in the contract
 * ("Events to the UI go over one Channel<DeviceEvent> per window,
 * re-subscribed when a destroyed window is recreated"). The channel is
 * created here and handed to `connect` as the `events` argument; the exact
 * argument name is a UI-side bridging choice, not yet fixed by the
 * contract, since only the command list is specified there.
 */
export function openDeviceEventChannel(onEvent: (event: DeviceEvent) => void): Channel<DeviceEvent> {
	const channel = new Channel<DeviceEvent>();
	channel.onmessage = onEvent;
	return channel;
}
