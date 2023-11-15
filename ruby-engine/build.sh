cargo build --release
rm -f lib/libyggdrasilffi.so
cp ../target/release/libyggdrasilffi.so lib/
gem build yggdrasil-engine.gemspec