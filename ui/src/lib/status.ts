// What the interface says about the device, in one place.
//
// The connection chip in the top bar and anything else that names the device's state read these,
// so the app cannot describe the same connection two different ways on two different surfaces.
// The tray (crates/hyperpace-app/src/tray.rs) derives the same facts on the Rust side, because it
// has to keep working with every window closed; the wording here is deliberately kept identical
// to that module's, so the tray and the window never disagree in front of the operator.
//
// Nothing here invents a state: with no reading, the caller is told there is none.

import type { DeviceIdentity } from './types';

/** Where the mouse is, as one value. */
export type Presence = 'preview' | 'missing' | 'asleep' | 'awake';

/** The fields these helpers read, so they can be called with the store or with a plain snapshot. */
export interface DeviceStatusSource {
	connected: boolean;
	online: boolean;
	identity: DeviceIdentity | null;
	/** The settings read from the mouse, when they have been; the polling rate comes from here. */
	settings: { pollingHz: number } | null;
}

/**
 * The polling rate the mouse is set to, as a label, or null before its settings have been read.
 *
 * Never the link's ceiling (`identity.maxPollingHz`): "8000 Hz" beside the link reads as the rate
 * the mouse is running at, and showed 8000 Hz to an operator who had just set 1000 Hz.
 */
function pollingLabel(device: DeviceStatusSource): string | null {
	return device.settings ? `${device.settings.pollingHz} Hz` : null;
}

/**
 * Where the mouse is. `preview` means the page is not running inside the desktop shell at all
 * (`npm run dev` in a browser), where no device can exist and saying "no mouse found" would
 * describe the wrong thing.
 */
export function presenceOf(device: DeviceStatusSource, inShell: boolean): Presence {
	if (!inShell) return 'preview';
	if (!device.connected) return 'missing';
	return device.online ? 'awake' : 'asleep';
}

/** One plain sentence naming what the mouse is doing right now. */
export function statusLine(device: DeviceStatusSource, inShell: boolean): string {
	switch (presenceOf(device, inShell)) {
		case 'preview':
			return 'Preview mode';
		case 'missing':
			return 'No mouse found';
		case 'asleep':
			return 'Mouse asleep';
		case 'awake': {
			const identity = device.identity;
			if (!identity) return 'Connected';
			const link = identity.wired ? 'Connected by cable' : 'Connected over 2.4 GHz';
			const polling = pollingLabel(device);
			return polling ? `${link}, polling at ${polling}` : link;
		}
	}
}

/** The short form of a presence, for the chip in the top bar. */
export function presenceLabel(presence: Presence): string {
	switch (presence) {
		case 'preview':
			return 'Preview';
		case 'missing':
			return 'No mouse';
		case 'asleep':
			return 'Asleep';
		case 'awake':
			return 'Connected';
	}
}

/**
 * The link and the polling rate beside the presence chip, or null when there is nothing true to
 * say: no connection, or a connection whose identity has not resolved yet. The rate is left off
 * until the settings have been read rather than filled with a stand-in.
 */
export function linkDetail(device: DeviceStatusSource, inShell: boolean): string | null {
	if (presenceOf(device, inShell) !== 'awake') return null;
	const identity = device.identity;
	if (!identity) return null;
	const link = identity.wired ? 'Wired' : '2.4 GHz';
	const polling = pollingLabel(device);
	return polling ? `${link} \u00b7 ${polling}` : link;
}

/** The tone a presence carries, for the chip that shows it. */
export function toneOf(presence: Presence): 'neutral' | 'ok' | 'warning' {
	switch (presence) {
		case 'awake':
			return 'ok';
		case 'asleep':
			return 'warning';
		case 'preview':
		case 'missing':
			return 'neutral';
	}
}
