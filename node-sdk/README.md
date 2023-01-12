# Node SDK

This SDK doesn't use FFI like the others, instead it compiles the underlying Rust engine to WASM. To do this you'll need `wasm-pack`.

## Development

To run the tests, first you'll need to link the WASM code to the JS test harness, so navigate to the `pkg` folder and run `yarn link`, then navigate back to the root of this SDK and run `yarn link unleash-node-core`, then install packages normally with yarn and finally run `yarn test`. To run the build, execute `yarn build`.