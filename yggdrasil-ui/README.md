# Yggdrasil UI

Interactive web playground for the Yggdrasil engine. It fetches feature definitions from an Unleash instance, exposes the compiled grammar produced by Yggdrasil, lets you edit that grammar, and evaluates toggles for arbitrary contexts – all inside the browser through WebAssembly.

## Prerequisites

- Node.js 18+ (or any runtime supported by Vite)
- npm (or pnpm/yarn if you prefer) for installing packages
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) to build the Rust WebAssembly bindings

## First-time setup

1. Build the WebAssembly bindings in development mode (writes artefacts to `src/wasm/` and patches the generated shim to use the local `env` stub):

   ```bash
   npm run wasm:dev
   ```

   Use `npm run wasm:build` for an optimised build.

2. Install frontend dependencies:

   ```bash
   npm install
   ```

3. Start the dev server:

   ```bash
   npm run dev
   ```

   Open the reported URL (default `http://localhost:5173`) in your browser.

If you regenerate the wasm bindings while `npm run dev` is running, Vite will hot-reload the new module automatically.

## Using the playground

1. Enter the Unleash client features endpoint and an optional API token, then fetch the configuration.
2. Inspect the grammar map (JSON) returned from the Yggdrasil upgrade step. Edit it and apply your overrides to recompile the rules inside the wasm engine.
3. Select any toggle, provide a context JSON payload, and evaluate to see the enabled state plus variant information.
4. Any compilation warnings from Yggdrasil are surfaced above the editor.

## Development notes

- The WebAssembly bindings live in the sibling crate `yggdrasil-wasm`. Unit tests for the Rust layer run via `cargo test -p yggdrasil-wasm`.
- `src/wasm/` is ignored by git because it contains generated output from `wasm-pack`.
- The TypeScript types for the wasm exports live in `src/wasm-types.d.ts`. Adjust them if you expose additional Rust functions.
