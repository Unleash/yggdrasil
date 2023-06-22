# PHP bindings to Yggdrasil

## Build

PHP relies on the C ABI and as such, we need to build the C headers as well.

See the README in the adjacent yggdrasilffi project for more information.

Once the C headers are built, you'll need to copy them across to this project, a small script is provided to do this:

```bash
./build.sh
```

This needs to be done every time changes are made to the yggdrasilffi project.

You'll also need to install the dependencies:

```bash
composer install
```


## Running the tests

You'll need to set the path to the Yggdrasil native library like so:

```bash
export YGGDRASIL_LIB_PATH=/home/{YOUR_NAME_HERE}/dev/yggdrasil/target/release
```

Then tests can be run with:

```bash
./vendor/bin/phpunit tests
```

