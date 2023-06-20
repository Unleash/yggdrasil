# Go bindings to Yggdrasil

## Build

Go requires a little fiddling to get access to the native library to work correctly.

Start by building the native library:

```bash
cargo build --release
```

You'll also need to generate the C header file, see the yggdrasilffi project for more information.

A small build script is provided to get the files into the right places:

```bash
./build.sh
```


## Running the tests

You'll need to set the path to the Yggdrasil native library like so:

```bash
export YGGDRASIL_LIB_PATH=/home/{YOUR_NAME_HERE}/dev/yggdrasil/target/release
```


And also tell Go how to search for the native lib for linking:

```bash
export LD_LIBRARY_PATH=/home/simon/dev/yggdrasil/go-engine:$LD_LIBRARY_PATH go test
```

