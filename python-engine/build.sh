cargo build --release
mkdir -p yggdrasil_engine/lib

[ -f ../target/release/yggdrasilffi.dll ] && cp ../target/release/yggdrasilffi.dll ./yggdrasil_engine/lib
[ -f ../target/release/libyggdrasilffi.dylib ] && cp ../target/release/libyggdrasilffi.dylib ./yggdrasil_engine/lib
[ -f ../target/release/libyggdrasilffi.so ] && cp ../target/release/libyggdrasilffi.so ./yggdrasil_engine/lib