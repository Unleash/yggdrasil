# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Yggdrasil is a multi-language SDK core for Unleash feature flags, written in Rust and providing bindings for multiple programming languages. The core Rust library (`unleash-yggdrasil`) implements the domain logic for feature flag evaluation, which is then exposed to various language ecosystems through FFI or WASM.

## Architecture

The project follows a hub-and-spoke architecture:

- **Core Rust Library** (`unleash-yggdrasil/`): Contains the main feature evaluation logic, strategy parsing, state management, and compiled toggle functionality
- **FFI Layer** (`yggdrasilffi/`): Provides C-compatible FFI bindings for native language integrations
- **WASM Layer** (`yggdrasilwasm/`, `pure-wasm/`): Compiles the core to WebAssembly for JavaScript/browser usage
- **Language Bindings**: Each language directory contains specific bindings and wrappers:
  - `dotnet-engine/`: .NET/C# bindings using FFI
  - `java-engine/`: Java bindings using JNA and WASM
  - `javascript-engine/`: Node.js/Bun bindings using WASM
  - `python-engine/`: Python bindings using PyO3
  - `ruby-engine/`: Ruby bindings using FFI
  - `go-engine/`: Go bindings using CGO
  - `php-engine/`: PHP bindings using FFI

## Common Development Commands

### Building the Core
```bash
# Build the main Rust library
cargo build --release

# Build specific components
cargo build -p unleash-yggdrasil --release
cargo build -p yggdrasilffi --release
```

### Testing
```bash
# Run all Rust tests
cargo test

# Run tests for specific package
cargo test -p unleash-yggdrasil
cargo test -p yggdrasilwasm

# Run benchmarks
cargo bench

# Run property-based grammar tests
cargo test -p unleash-yggdrasil grammar_prop_tests
```

### Language-Specific Commands

#### JavaScript/Node.js
```bash
cd javascript-engine/
bun build  # or npm run build
bun test   # or npm test
```

#### Java
```bash
cd java-engine/
./gradlew test
./gradlew build
./gradlew spotlessApply  # Format code
```

#### .NET
```bash
cd dotnet-engine/
dotnet build
dotnet test
dotnet pack  # Create NuGet package
```

#### Python
```bash
cd python-engine/
# Using poetry (preferred)
poetry install
poetry run pytest tests/

# Or using pip/tox
python -m pip install -r requirements.txt
python -m pytest tests/
tox  # Run tests across Python versions

# Build wheel for distribution
./build.sh
```

#### Ruby
```bash
cd ruby-engine/
bundle install
bundle exec rspec

# Build gem for distribution
./build.sh
```

#### Go
```bash
cd go-engine/
go test
go build

# Build with CGO bindings
./build.sh
```

#### PHP
```bash
cd php-engine/
composer install
composer test  # Runs PHPUnit tests

# Build FFI bindings
./build.sh
```

### WASM Development
```bash
# Build WASM for JavaScript
cargo build --target wasm32-unknown-unknown --release -p yggdrasilwasm

# Build pure WASM (for Java integration)
cd pure-wasm/
cargo build --target wasm32-unknown-unknown --release

# Build WASM from Java engine (includes WASM compilation)
cd java-engine/
./gradlew buildWasm

# Test WASM in JavaScript environment
cd yggdrasilwasm/e2e-tests/
bun test
```

## Key Concepts

- **CompiledToggle**: The core data structure representing a feature flag with its evaluation rules
- **Strategy Parsing**: Custom DSL parser using Pest grammar for complex rollout strategies
- **State Management**: Efficient state representation using DashMap and AHashMap for concurrent access
- **FFI Safety**: All FFI functions use sendable closures and proper memory management
- **WASM Optimization**: Separate WASM builds for different use cases (browser vs server)

## Development Workflow

1. **Core Changes**: Make changes to `unleash-yggdrasil/src/` first
2. **Test Core**: Run `cargo test -p unleash-yggdrasil` 
3. **Update Bindings**: Update relevant language bindings if API changes
4. **Test Language Bindings**: Run language-specific tests
5. **Integration Testing**: Use `test-data/` JSON files for cross-language compatibility tests

## File Structure Notes

- `flat-buffer-defs/`: Protocol definitions for efficient serialization
- `test-data/`: Shared test fixtures used across all language implementations
- Each language engine includes its own README with specific setup instructions
- Version management is handled independently per language to match ecosystem conventions
- `Cross.toml`: Configuration for cross-compilation to different targets
- `devenv.nix`/`devenv.yaml`/`devenv.lock`: Development environment setup using devenv

## Cross-Platform Build Support

The project uses Cross.toml for building native libraries across different platforms:
```bash
# Install cross if not already installed
cargo install cross

# Build for specific target (example for Linux x86_64)
cross build --target x86_64-unknown-linux-gnu --release
```

## Development Environment

The project includes devenv configuration for reproducible development environments:
```bash
# Using devenv (if installed)
devenv shell

# This will provide all necessary tools including Rust, Node.js, etc.
```