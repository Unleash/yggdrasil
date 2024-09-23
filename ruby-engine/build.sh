#!/bin/bash
set -e

cargo build --release

## Start copy the correct binary to the correct name
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin)
    EXT="dylib"
    ;;
  linux)
    EXT="so"
    ;;
  msys*|mingw*|cygwin*)
    EXT="dll"
    ;;
  *)
    echo "Unsupported platform: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64)
    ARCH_SUFFIX="x86_64"
    ;;
  arm*|aarch64)
    ARCH_SUFFIX="arm"
    ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

LIB_NAME="libyggdrasilffi_${ARCH_SUFFIX}.${EXT}"

rm -f "lib/$LIB_NAME"
cp "../target/release/libyggdrasilffi.${EXT}" "lib/$LIB_NAME"
## End copy the correct binary to the correct name

# Build the gem
gem build yggdrasil-engine.gemspec