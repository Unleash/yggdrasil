using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Running;
using Yggdrasil;

public class EngineBenchmarks
{
    private YggdrasilEngine engine;
    private Context context;

    public EngineBenchmarks()
    {
        engine = new YggdrasilEngine();
        context = new Context();
    }

    [Benchmark]
    public void EmptyIsEnabled() => engine.IsEnabled("missing", context);
}

class Program
{
    static void Main(string[] args)
    {
        var summary = BenchmarkRunner.Run<EngineBenchmarks>();
    }
}