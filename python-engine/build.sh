cargo build --release
mkdir -p lib

[ -f ../target/release/yggdrasilffi.dll ] && cp ../target/release/yggdrasilffi.dll ./lib
[ -f ../target/release/libyggdrasilffi.dylib ] && cp ../target/release/libyggdrasilffi.dylib ./lib
[ -f ../target/release/libyggdrasilffi.so ] && cp ../target/release/libyggdrasilffi.so ./lib