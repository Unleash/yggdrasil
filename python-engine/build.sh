cargo build --release
mkdir -p yggdrasil-engine/lib

[ -f ../target/release/yggdrasilffi.dll ] && cp ../target/release/yggdrasilffi.dll ./yggdrasil-engine/lib
[ -f ../target/release/libyggdrasilffi.dylib ] && cp ../target/release/libyggdrasilffi.dylib ./yggdrasil-engine/lib
[ -f ../target/release/libyggdrasilffi.so ] && cp ../target/release/libyggdrasilffi.so ./yggdrasil-engine/lib