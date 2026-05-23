# Yggdrasil

![world tree image](worldtree.webp 'Title')

##### One SDK core connecting all the realms of Unleash.

Yggdrasil is a Rust project designed to create the core of the Unleash SDK domain logic in a single language (in this case Rust).

## Building the Core

Easy enough - run `cargo build --release` from the root of the project. You'll need an up to date set of Rust tools to do this.

To run the client specs, you'll first need to clone them:

`git clone --depth 5 --branch v5.1.9 https://github.com/Unleash/client-specification.git client-specification`

## Testing

This will run whole test suite

```
cargo test
```

## Node

The Node core is a special case, this doesn't use FFI like the other SDKs, instead this compiles the core down to WASM.

### Building

You'll need to build the core first, once that's compiled, from the root of the `node-sdk` project, run `wasm-pack build`. Once that's built, navigate to `{root}/node-sdk/pkg` and run `npm link`, which should make the node package available. Finally, from `{root}/node-sdk/www` run

```
npm link node-sdk
npm install
npm start
```

You can open a browser and head to `http://localhost:8080` and inspect the console to see the Unleash engine evaluate a toggle.

## Yggdrasil Playground (Web UI)

The `yggdrasil-ui` package hosts a browser-based playground that runs the Yggdrasil engine via WebAssembly and lets you:

- Fetch feature definitions from an Unleash instance (your own deployment or the Unleash sandbox).
- Inspect and tweak the compiled grammar that Yggdrasil generates for each toggle.
- Evaluate toggles against arbitrary context payloads directly in the browser.

### Quick start

From the repository root:

1. Build the WebAssembly bundle in development mode (writes artefacts into `yggdrasil-ui/src/wasm/`):

   ```bash
   (cd yggdrasil-ui && npm run wasm:dev)
   ```

   Re-run this command whenever you change the Rust source in `yggdrasil-wasm/`. Use `npm run wasm:build` for an optimised release build.

2. Install frontend dependencies and start the dev server:

   ```bash
   (cd yggdrasil-ui && npm install && npm run dev)
   ```

   Vite serves the UI on `http://localhost:5173` by default.

3. In the browser:

   - To use the public Unleash sandbox, point the form at `http://localhost:5173/unleash-sandbox/enterprise/api/client/features`. The Vite dev server proxies that path to `https://sandbox.getunleash.io`, avoiding CORS issues.
   - Alternatively, supply the URL to any Unleash instance reachable from your machine (e.g. a local Unleash server on another port).
   - Provide an API token if your endpoint requires it, fetch the features, and explore the “Compiled Grammar” and “Evaluate Toggle” sections.

More details about the playground live in `yggdrasil-ui/README.md`.

## Python

The Python core uses [PyO3](https://pyo3.rs/v0.17.2/index.html) for its bindings.

### Building

Start by setting up and activating a virtual environment in the python-sdk folder:

```
python3 -m venv venv
source venv/bin/activate
```

Install [maturin](https://github.com/PyO3/maturin) by executing `pip install maturin` in your shell. Then you can run `maturin develop`.
