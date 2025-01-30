```

BenchmarkDotNet v0.14.0, Ubuntu 22.04.5 LTS (Jammy Jellyfish)
13th Gen Intel Core i7-13700HX, 1 CPU, 24 logical and 16 physical cores
.NET SDK 8.0.112
  [Host]     : .NET 8.0.12 (8.0.1224.60305), X64 RyuJIT AVX2
  DefaultJob : .NET 8.0.12 (8.0.1224.60305), X64 RyuJIT AVX2


```
| Method         | Mean     | Error   | StdDev  |
|--------------- |---------:|--------:|--------:|
| EmptyIsEnabled | 900.7 ns | 8.37 ns | 7.83 ns |
