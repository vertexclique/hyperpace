URL: https://v2.tauri.app/start/frontend/sveltekit/
Additional URLs: https://v2.tauri.app/start/frontend/vite/ ; https://github.com/tauri-apps/create-tauri-app/tree/dev/templates/template-svelte-ts (commit 890e6208661617b2e86a21f662b6efe9e3035788, 2026-09-09)
Retrieved: 2026-09-15
Source type: official documentation and official scaffolding templates (primary)

## Verbatim, SvelteKit page

> SvelteKit is a meta framework for Svelte. Learn more about SvelteKit at https://svelte.dev/. This guide is accurate as of SvelteKit 2.20.4 / Svelte 5.25.8.

> ## Checklist
>
> - Use [SSG](https://svelte.dev/docs/kit/adapter-static) and [SPA](https://svelte.dev/docs/kit/single-page-apps) via `static-adapter`. Tauri doesn't support server-based solutions.
> - If using SSG **with prerendering**, be aware that `load` functions will not have access to tauri APIs during the build process of your app. Using SPA mode (without prerendering) is recommended since the load functions will only run in the webview with access to tauri APIs.
> - Use `build/` as `frontendDist` in `tauri.conf.json`.

```json
    {
      "build": {
        "beforeDevCommand": "npm run dev",
        "beforeBuildCommand": "npm run build",
        "devUrl": "http://localhost:5173",
        "frontendDist": "../build"
      }
    }
```

## Verbatim, Vite page

> Vite is a build tool that aims to provide a faster and leaner development experience for modern web projects.
> This guide is accurate as of Vite 8.

> - Use `../dist` as `frontendDist` in `src-tauri/tauri.conf.json`.
> - Use `process.env.TAURI_DEV_HOST` as the development server host IP when set to run on iOS physical devices.

## Directory listing, tauri-docs `start/frontend/`

```
index.mdx leptos.mdx nextjs.mdx nuxt.mdx qwik.mdx sveltekit.mdx trunk.mdx vite.mdx
```

## create-tauri-app templates containing "svelte"

```
template-svelte
template-svelte-ts
```

## Verbatim, template-svelte-ts/package.json.lte

```
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "prepare": "svelte-kit sync || echo ''",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "tauri": "tauri"
  },
  "license": "MIT",
  "dependencies": {
    "@tauri-apps/api": "{% if v2 %}^2{% else %}^1{% endif %}"{% if v2 %},
    "@tauri-apps/plugin-opener": "^2"{% endif %}
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.10",
    "@sveltejs/kit": "^2.65.1",
    "@sveltejs/vite-plugin-svelte": "^7.1.2",
    "svelte": "^5.56.3",
    "svelte-check": "^4.6.0",
    "typescript": "~6.0.3",
    "vite": "^8.0.16",
    "@tauri-apps/cli": "{% if v2 %}^2{% else %}^1{% endif %}"
  }
```

## Verbatim, template-svelte-ts/svelte.config.js

```js
// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: "index.html",
    }),
  },
};
```
