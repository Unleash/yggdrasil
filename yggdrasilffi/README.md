# Yggdrasil FFI Layer

A single FFI layer to the Yggdrasil project. This allow us to use the library in a range of languages.

## Build

This will be built by the main project, using the standard cargo build command.

```bash
cargo build
```

## Generate Headers

For languages that need the C ABI, like PHP and Go, you can use cbindgen to generate the headers.


```bash
cargo install --force cbindgen

cbindgen --config cbindgen.toml --lang c --crate yggdrasilffi --output unleash_engine.h
```
