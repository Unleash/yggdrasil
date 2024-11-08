# Python Bindings to Yggdrasil

Provides high level bindings to the Unleash Yggdrasil engine.

## Build and test
This project uses [poetry](https://python-poetry.org/).

Before you begin, you'll need to setup the native library. You'll need a Rust compiler. If you're on Windows, you'll need bash or just read the script and do the equivalent powershell steps.

``` sh
./build.sh
```

To run tests:

```poetry run pytest```

For local development, it can be convenient to have a shell to work in:

```poetry shell```

## Publish

Publishing is done through Github. Ensure you've bumped the version in `yggdrasil-engine/__init__.py`. Note that yggdrasilCoreVersion in the same will, will determine what version of the native libraries are resolved for the build; the build does not work against the Rust source code directly.