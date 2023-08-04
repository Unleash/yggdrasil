# Ruby Bindings to Yggdrasil

## Running the tests

First make sure that have built the native FFI code. That can be done with Cargo anywhere in this project:


```bash
cargo build --release
```

You'll also need to set the path to the Yggdrasil native library like so:

```bash
export YGGDRASIL_LIB_PATH=/home/{YOUR_NAME_HERE}/dev/yggdrasil/target/release
```

Then you can run the tests with:

```bash
rspec
```

## Build

You can build the gem with:

```bash
gem build unleash-engine.gemspec

```

Then you can install the gem for local development with:

```
gem install unleash-engine-0.0.1.gem
```
