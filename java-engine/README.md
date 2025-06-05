# Java Bindings to Yggdrasil

## Generate flatbuffers

You need flatbuffers compiler version 23.1.21

```bash
flatc --java -o java-engine/src/main/java flat-buffer-defs/enabled-message.fbs
```

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
