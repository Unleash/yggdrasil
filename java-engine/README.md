# Java Bindings to Yggdrasil

## Build

We use gradle here:

```bash
./gradlew build
```

## Running the tests

You'll need to set the path to the Yggdrasil native library like so:

```bash
export YGGDRASIL_LIB_PATH=/home/{YOUR_NAME_HERE}/dev/yggdrasil/target/release
```

Then tests can be run with:

```bash
./gradlew test
```