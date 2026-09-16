// Client-only rendering: the Tauri shell has no server, so every route is
// rendered in the webview from the static fallback (see svelte.config.js).
export const ssr = false;
