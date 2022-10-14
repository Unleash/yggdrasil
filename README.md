# Yggdrasil

One SDK core connecting them all.

Yggdrasil is a Rust project designed to create the core of the Unleash domain logic in a single language (in this case Rust).

## Building the Core

Easy enough - run `cargo build` from the root of the project. You'll need an up to date set of Rust tools to do this.

## Node

The Node core is a special case, this doesn't use FFI like the other SDKs, instead this compiles the core down to WASM.

### Building

You'll need to build the core first, once that's compiled, from the root of the `node-sdk` project, run `wasm-pack build`. Once that's built, navigate to `{root}/node-sdk/pkg` and run `npm link`, which should make the node package available. Finally, from `{root}/node-sdk/www` run

- `npm link 'node-sdk'`
- `npm install`
- `npm start`

You can open a browser and head to `http://localhost:8080` and inspect the console to see the Unleash engine evaluate a toggle.
