# SUMMARY: tauri-2026-svelte-frontend-setup

## Claim
- The official create-tauri-app Svelte templates (`template-svelte`, `template-svelte-ts`) are SvelteKit apps using `@sveltejs/adapter-static` with `fallback: "index.html"` (SPA mode).
- They pin svelte ^5.56.3, kit ^2.65.1, vite ^8.0.16 and TypeScript ~6.0.3.
- The SvelteKit guide recommends SPA mode without prerendering, with `frontendDist: ../build`.
- A plain Vite guide also exists (`frontendDist: ../dist`).

## Method
The official frontend guides and the template files in create-tauri-app (commit 890e6208).

## Result
Neither of the two Svelte templates is plain Svelte plus Vite without Kit.

## Evidence tier
1.

## Performance
Not measured.

## Correctness
The SvelteKit page says it is accurate as of SvelteKit 2.20.4 / Svelte 5.25.8, which is older than the template pins.

## Relevance to hyperpace
Two options for the operator:
- **SvelteKit SPA:** the official template, with routing included and more tooling.
- **Plain Svelte 5 plus Vite:** fewer dependencies, enough for a few settings screens.

Both embed identically.
