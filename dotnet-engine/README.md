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

## Development

You can publish local packages to test with your SDK like this:

```bash
dotnet build
dotnet pack /p:Version=1.0.0-alpha.0
cd bin/Debug
dotnet nuget push "*.nupkg" -s ~/path/to/local/feed
```

Then add that local folder as a feed in NuGet

```bash
dotnet nuget add source ~/path/to/local/feed
```

Now you can switch package source in package manager and import your locally published package to work with.