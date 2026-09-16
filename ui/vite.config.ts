import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Root-relative asset paths (SvelteKit's default) resolve correctly under
// Tauri's asset protocol, which serves the built `build/` directory as the
// app's origin root. The fixed dev port matches what a future
// crates/hyperpace-app tauri.conf.json would point `devUrl` at.
export default defineConfig({
	plugins: [sveltekit()],
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true
	},
	build: {
		target: 'es2022'
	}
});
