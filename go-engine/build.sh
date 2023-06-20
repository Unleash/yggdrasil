# Copy the native library and header file across if they don't exist

cp ../yggdrasilffi/unleash_engine.h .
cp ../target/release/libyggdrasilffi.so .

export LD_LIBRARY_PATH="${YGGDRASIL_LIB_PATH}:${LD_LIBRARY_PATH}"

go test