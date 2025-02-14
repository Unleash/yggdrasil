using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Running;
using Yggdrasil;

public class EngineBenchmarks
{
    private YggdrasilEngine engine;
    private Context context;
    private Context fatContext;

    public EngineBenchmarks()
    {
        engine = new YggdrasilEngine();
        context = new Context();
        fatContext = new Context();
        fatContext.UserId = "constrainKey";
        fatContext.AppName = "constrainValue";
        fatContext.Properties = new Dictionary<string, string>
        {
            { "constrainKey", "constrainValue" }
        };
    }

    [Benchmark]
    public void EmptyIsEnabled() => engine.IsEnabled("Feature.A", context).Enabled();

    [Benchmark]
    public void EmptyIsEnabledWithBigContext() => engine.IsEnabled("Feature.A", fatContext).Enabled();
}

class Program
{
    static void Main(string[] args)
    {
        var summary = BenchmarkRunner.Run<EngineBenchmarks>();
    }
}
