# Ruby Bindings to Yggdrasil

## Running the tests

First make sure that you have built the native FFI code and it's located in the right place. This can be done with the build script in `build.sh`:


```bash
./build.sh
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
