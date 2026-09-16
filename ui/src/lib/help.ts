// Every explanation a "?" beside a control shows, in one place.
//
// One catalogue for the whole app, so a setting is explained the same way wherever it appears.
// Each entry says what the control does to the hardware and what the tradeoff is, in plain words.
// Where the behaviour is established by how the firmware stores the value (the protocol reference,
// docs/research/mouse-protocol-v2.md) or by the vendor's own configurator text, it is stated as
// fact. Where it is not confirmed, the entry says so in its note instead of guessing: an
// explanation that invents a behaviour is worse than none.

export interface HelpEntry {
	/** The control's name, as the explanation's heading. */
	title: string;
	/** What it does and why someone would change it. */
	body: string;
	/** A caveat, a limit, or what is not confirmed. */
	note?: string;
}

export const HELP = {
	// Performance: DPI.
	dpiStages: {
		title: 'DPI stages',
		body: 'A list of sensitivities, up to eight. The DPI switch button steps through them, and the highlighted stage is the one in use now. Higher DPI moves the pointer further for the same hand movement.',
		note: 'Each stage keeps its own colour, which the DPI indicator shows when you switch.'
	},
	dpiValue: {
		title: 'Stage sensitivity',
		body: 'Counts per inch for this stage: how far the pointer travels for one inch of mouse movement. Most people aim between 400 and 3200 and adjust the operating system pointer speed to taste.',
		note: 'Values are rounded to the steps this sensor supports.'
	},
	dpiColor: {
		title: 'Stage colour',
		body: 'The colour the DPI indicator shows when this stage becomes active, so you can tell stages apart without opening the app.'
	},
	dpiAddStage: {
		title: 'Add stage',
		body: 'Adds another sensitivity to the cycle the DPI switch button steps through. Up to eight stages.'
	},

	// Performance: tracking.
	pollingRate: {
		title: 'Polling rate',
		body: 'How many times per second the mouse reports its position. Higher rates cut input delay and make fast movement smoother, but use more battery and more computer processing.',
		note: 'The choices offered are the ones the current connection supports. Some games and older computers stutter above 1000 Hz.'
	},
	liftOff: {
		title: 'Lift-off distance',
		body: 'How high the mouse can be lifted before the sensor stops tracking. A lower distance stops the pointer drifting when you lift and reposition the mouse; a higher one tolerates uneven or thick surfaces.'
	},
	sensorMode: {
		title: 'Sensor mode',
		body: 'Low power runs the sensor more slowly to make the battery last longer. High performance runs it at full speed for the most responsive tracking.'
	},
	debounce: {
		title: 'Debounce time',
		body: 'The minimum time between two registered clicks on the same button. A shorter time makes rapid clicks register faster; too short and a single press can register as a double click.',
		note: 'If you see accidental double clicks, raise this until they stop.'
	},

	// Performance: motion.
	motionSync: {
		title: 'Motion sync',
		body: 'Times each sensor reading to line up with the moment the computer asks for it, so every report carries a fresh reading. Tracking becomes more consistent, at the cost of a very small added delay that shrinks at higher polling rates.'
	},
	angleSnap: {
		title: 'Angle snap',
		body: 'Straightens nearly horizontal and vertical movements, which helps drawing straight lines. It also distorts diagonal and small corrective movements, so it is usually left off for games.'
	},
	ripple: {
		title: 'Ripple control',
		body: 'Smooths out small jitter in the sensor reading, most noticeable at very high DPI. The smoothing adds a slight softness to movement, so leave it off unless the pointer shakes when the mouse is still.'
	},
	longRange: {
		title: 'Long range mode',
		body: 'Strengthens the wireless link so the mouse holds its connection further from the receiver and through more interference. It shortens battery life.',
		note: 'Only applies over the 2.4 GHz receiver, not by cable.'
	},

	// Performance: power.
	sleepTime: {
		title: 'Sleep after',
		body: 'How long the mouse waits without movement before it sleeps to save battery. Moving it wakes it again, which takes a moment, so a very short time can make the first movement after a pause feel late.'
	},
	highestPerformance: {
		title: 'Highest performance',
		body: 'Keeps the mouse at full power for the most responsive tracking. The timer sets how long it stays there before stepping down to save battery.',
		note: 'This drains the battery noticeably faster while it is active.'
	},

	// Buttons.
	buttonBinding: {
		title: 'Button action',
		body: 'What this button does when pressed: a mouse click, a DPI switch, horizontal scrolling, a key combination, a media key, a macro, a polling rate switch, or nothing at all.',
		note: 'Left click must stay assigned to one of the buttons, so the mouse can always click.'
	},
	fireKey: {
		title: 'Fire key',
		body: 'Repeats a click from one press. Times sets how many repeats, from 0 to 3, where 0 keeps repeating for as long as you hold the button. Interval sets the pause between repeats, from 10 to 255.',
		note: 'The unit of the interval is not confirmed by the vendor; milliseconds is the likely reading.'
	},
	keystroke: {
		title: 'Key combination',
		body: 'Sends a keyboard shortcut, for example Ctrl and C, when the button is pressed. Pick the modifiers and one key.'
	},
	pollingSwitch: {
		title: 'Polling rate switch',
		body: 'Makes the button step through polling rates, so you can change the rate on the fly without opening the app.'
	},

	// Macros.
	macroRecord: {
		title: 'Record',
		body: 'Captures the keys and clicks you press, with the time between them, as a sequence you can bind to a button.'
	},
	macroDelay: {
		title: 'Delays',
		body: 'The pause before each step. Recorded delays keep your real timing; a default delay replaces them with one fixed pause.',
		note: 'The unit the mouse uses for macro delays is not confirmed by the vendor.'
	},
	macroCycle: {
		title: 'Repeat',
		body: 'How the macro repeats: a set number of times, until the button is released, until any button is pressed, or until the same button is pressed again.'
	},

	// Lighting: DPI indicator and receiver.
	dpiIndicator: {
		title: 'DPI indicator',
		body: 'The light that shows the active DPI stage in that stage colour. Its effect sets how it shows it: off, steady, or breathing.'
	},
	lightBrightness: {
		title: 'Brightness',
		body: 'How bright the light is. Lower brightness uses less battery.'
	},
	lightSpeed: {
		title: 'Speed',
		body: 'How fast an animated effect such as breathing or rainbow moves. It has no visible effect on a steady colour.'
	},
	receiverLight: {
		title: 'Receiver light',
		body: 'The light on the 2.4 GHz receiver itself: its effect, colour, speed and brightness. The receiver stores it, so it stays as set when the mouse sleeps or the app is closed.'
	},
	receiverEffect: {
		title: 'Light effect',
		body: 'The pattern the light plays: off, a fixed colour, a single colour breathing, rainbow, rainbow breathing, or neon. Colour applies only to the single colour effects.'
	},
	lightColor: {
		title: 'Colour',
		body: 'The colour for the fixed and breathing effects. Choose from the preset swatches, drag the colour bar, or type an exact hex or red, green and blue value.'
	},
	receiverTime: {
		title: 'Time',
		body: 'A timing value the receiver stores with its light settings.',
		note: 'Its unit and exact effect are not confirmed by the vendor, so change it only if you want to experiment.'
	},
	pairing: {
		title: 'Pairing',
		body: 'Links a mouse to this receiver. Press left, right and the wheel together for 3 seconds until the mouse light flashes, keep it close to the receiver, then start pairing.'
	},
	factoryReset: {
		title: 'Factory reset',
		body: 'Returns every setting stored on the mouse to its factory default: DPI stages, buttons, polling rate and the rest. Macros saved in this app are not deleted.',
		note: 'This cannot be undone. Save a profile first if you might want your settings back.'
	},

	// Settings.
	profile: {
		title: 'On-board profile',
		body: 'The mouse stores separate sets of settings in slots. Switching slots changes every setting at once, and the choice is kept on the mouse, so it works on any computer.'
	},
	configExport: {
		title: 'Export and import',
		body: 'Saves every mouse setting to a file, or loads one back onto the mouse. Useful for backups and for copying a setup to another mouse of the same model.'
	},
	launchAtLogin: {
		title: 'Launch at login',
		body: 'Starts Hyperpace in the tray when you log in, so battery warnings and the tray status work without opening the app.'
	},
	lowBatteryWarning: {
		title: 'Low battery warning',
		body: 'Shows a notification when the battery drops to this percentage. It appears once per discharge and resets when you charge.'
	},
	firmwareWatch: {
		title: 'Watch for firmware',
		body: 'Checks now and then whether a new firmware release has been published for this mouse, and tells you if one has. It never downloads or installs anything by itself.'
	},
	rendererWorkaround: {
		title: 'Renderer workaround',
		body: 'Works around a display bug with some graphics drivers on Linux that leaves the window blank. Automatic turns it on only where that bug applies.',
		note: 'Takes effect the next time the app starts.'
	},

	// Firmware.
	firmwareArchive: {
		title: 'Firmware archive',
		body: 'Firmware packages kept on this computer for the mouse and the receiver, so you can install a newer version or return to an older one.',
		note: 'Installing firmware rewrites the device itself. Keep the mouse charged and connected until it finishes.'
	},
	firmwareImport: {
		title: 'Import a package',
		body: 'Adds a firmware package file to the archive. Only packages that match this mouse or receiver are accepted.'
	},

	// Data management.
	dataHistory: {
		title: 'History',
		body: 'A log of what happened to the mouse: connections, disconnections, sleep, settings changes it reported, and low battery. The newest 2000 entries are kept.'
	},
	dataProfiles: {
		title: 'Saved profiles',
		body: 'Snapshots of every mouse setting, stored on this computer. Restoring one writes all of its settings back to the mouse.'
	},
	dataBackup: {
		title: 'Backup',
		body: 'Exports everything Hyperpace stores on this computer (history, saved profiles, macros, settings and the firmware index) to one file, or merges a backup file back in.',
		note: 'Firmware package files themselves are not included in the backup.'
	}
} as const satisfies Record<string, HelpEntry>;

/** The name of one explanation in the catalogue. */
export type HelpKey = keyof typeof HELP;
