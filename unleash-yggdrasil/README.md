# Yggdrasil

Yggdrasil is a Rust project designed to create the core of the Unleash SDK domain logic in a single language. This crate is the Rust logic extracted into a standalone crate so that the logic can be used natively inside a Rust application. This is a very experimental crate, so if you're looking to connect a Rust application to Unleash, you should use the official [Rust SDK](https://crates.io/crates/unleash-api-client) instead, the API and philosophy here are subject to change until this is in stable.

## Using Yggdrasil in a Rust project

The heart of Yggdrasil is the `EngineState` struct, this is a lightweight struct that does very little on its own, it's cheap to create but you probably only need/want one of them:

``` rust

let engine = EngineState::default();

```

The engine needs to be populated with an Unleash response before it will return anything useful. The shape of the data here should be identical to the response returned by Unleash, if you'd like to handroll your own data, you can check out the format of the `ClientFeatures` struct [here](https://github.com/Unleash/unleash-types-rs/blob/main/src/client_features.rs).

```rust

let spec_data = "";// Some json blob, matching the Unleash format

let unleash_data: ClientFeatures = serde_json::from_str(&spec_data).unwrap();

engine.take_state(unleash_data);
```

Now you can query the engine for a given toggle:

```rust
let context = InnerContext::default();

let enabled = engine.is_enabled("my-toggle-name", &context);

//Do something with the enabled state here

```

# Releasing

Template
```
cargo smart-release -u -b {{patch/minor/major}} unleash-yggdrasil --execute
```
Example
```
cargo smart-release -u -b patch unleash-yggdrasil --execute
```
