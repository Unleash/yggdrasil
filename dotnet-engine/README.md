# CSharp Bindings for Yggdrasil

## Build

Build the base project with cargo:

```bash
cargo build --release
```

Csharp doesn't require the library path so this should work:

```bash
dotnet build
```

## Running the tests

```bash
dotnet test
```

## Running the benchmarks

```bash
dotnet run --project Yggdrasil.Benchmarks -c Release
```

Output can be read in Yggdrasil.Benchmarks/BenchmarkDotNet.Artifacts/results
