# Yggdrasil Engine (Pure WASM)

This project contains bindings for Yggdrasil that are suitable for compilation into a pure, standalone WebAssembly module. It’s designed for high portability and **does not require WASI**.

See the [Yggdrasil Java Engine](../java-engine) for an example integration.

## Development

### Working with Data Exchange Code

> [!IMPORTANT]
> Requires [flatc version 23.1.21](https://github.com/google/flatbuffers/releases/tag/v23.1.21)

We use [FlatBuffers](https://flatbuffers.dev/) for efficient data exchange over the WASM boundary. If you're modifying the data models, you’ll need to regenerate the FlatBuffer bindings:

``` bash
flatc --rust -o pure-wasm/src flat-buffer-defs/enabled-message.fbs
```

### Building

This project uses low-level bit-hacking optimizations that rely on a 32-bit architecture, so the only supported target is wasm32-unknown-unknown. If you don't have the WASM target installed you can do that with rustup:

``` bash
rustup target add wasm32-unknown-unknown
```

Build configuration is already defined in the root project's config.toml, so you can compile the WASM module with:

``` bash
cargo build --release --target wasm32-unknown-unknown
```

### Randomness Implementation

Yggdrasil requires a source of entropy for gradual rollout calculations. Since WASM modules lack built-in randomness, the host environment must provide it. When mounting this module in a runtime, you must supply the following host function:

``` rust
fn fill_random(ptr: *mut u8, len: usize) -> i32;
```

An example implementation can be found in the [Java integration](../java-engine/src/main/java/io/getunleash/engine/WasmInterface.java).