# Python Bindings to Yggdrasil

## Build

You'll need to setup a Python virtual environment:

``` bash
python3 -m venv venv
source venv/bin/activate
```

and install the dependencies:

``` bash
pip install -r requirements.txt
```


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
pytest
```
