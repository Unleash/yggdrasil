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

There's also a `mem_check.rb` in the scripts folder. This is not a bullet proof test, but it can be helpful for detecting large leaks. This requires human interaction - you need to read the output and understand what it's telling you, so it's not run as part of the test suite.

```bash
ruby scripts/mem_check.rb
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
