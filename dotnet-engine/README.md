# Unleash Yggdrasil .NET Engine

Unleash Yggdrasil Engine for .NET.

This is a .NET wrapper containing the core of the Unleash SDK domain logic, enabling you to develop your own Unleash .NET SDK.
If you're looking for the ready-to-use Unleash .NET SDK instead, you can find it [here](https://github.com/Unleash/unleash-client-dotnet).

Read more about Unleash at: https://www.getunleash.io/

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

The current target is .NET 6.0, so in order to run the tests you should have the respective runtime installed: https://dotnet.microsoft.com/en-us/download/dotnet/6.0/runtime

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

```bash
dotnet add package Yggdrasil.Engine --prerelease
```

Whenever you update your package you should:

```bash
# On the .nupkg folder
dotnet nuget push "*.nupkg" -s ~/path/to/local/feed
# On the project where it's used
dotnet nuget locals all --clear
dotnet restore
```
