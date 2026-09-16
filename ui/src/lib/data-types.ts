// Mirrors the Rust DTOs in crates/hyperpace-app/src/dto/data.rs field-for-field, the same
// convention types.ts documents for the rest of the app: every field name here is that struct's
// serde `rename_all = "camelCase"` name. This file backs the Data Management screen only; it does
// not redeclare a shape types.ts (or device.svelte.ts) already owns, e.g. macros and firmware
// entries keep using MacroRecord/FirmwareRecord from types.ts, read from the shared `device` store.

/// Mirrors DataOverviewDto: one collection count per field, plus the store's own location and
/// size on disk.
export interface DataOverview {
	macros: number;
	profiles: number;
	settings: number;
	firmware: number;
	events: number;
	storeRoot: string;
	storeSizeBytes: number;
}

/// Mirrors EventRecordDto. `at` is unix seconds.
export interface EventLogEntry {
	id: string;
	at: number;
	kind: string;
	message: string;
}

/// Mirrors ListEventsResponse: one page of the log, newest first, plus the log's total size so a
/// screen can page it without loading more than one page at a time.
export interface EventLogPage {
	events: EventLogEntry[];
	total: number;
}

/// Mirrors ProfileSnapshotDto. `capturedAt` is unix seconds.
export interface ProfileSnapshotRecord {
	id: string;
	name: string;
	index: number;
	capturedAt: number;
}

/// Mirrors ImportStoreResponse: how many records `import_store` actually added per collection (a
/// record already present in the destination store, by exact field equality, is skipped and not
/// counted here).
export interface ImportStoreResult {
	macrosAdded: number;
	profilesAdded: number;
	settingsAdded: number;
	firmwareAdded: number;
	eventsAdded: number;
}

/// Event log page size this screen asks for; `list_events` itself caps a request at 200
/// regardless of what is asked (`commands::data::EVENT_PAGE_LIMIT`).
export const EVENT_LOG_PAGE_SIZE = 50;
