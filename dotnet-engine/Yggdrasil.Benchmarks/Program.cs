using System;
using System.Text.Json;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Running;
using BenchmarkDotNet.Configs;
using BenchmarkDotNet.Columns;
using BenchmarkDotNet.Running;
using Newtonsoft.Json.Linq;
using Yggdrasil;

[Config(typeof(Config))]
public class YggBench
{
    private YggdrasilEngine yggdrasilEngine;

    private class Config : ManualConfig
    {
        public Config()
        {
            AddColumn(BenchmarkDotNet.Columns.StatisticColumn.OperationsPerSecond);
        }
    }

    [GlobalSetup]
    public void Setup()
    {
        var basePath = Path.Combine(
            "..",
            "..",
            "..",
            "..",
            "..",
            "..",
            "..",
            "..",
            "..",
            "client-specification",
            "specifications"
        );
        var suitePath = Path.Combine(basePath, "01-simple-examples.json");
        var suiteData = JObject.Parse(File.ReadAllText(suitePath));

        yggdrasilEngine = new YggdrasilEngine();
        yggdrasilEngine.TakeState(suiteData["state"].ToString());
    }

    [Benchmark]
    public void IsFeatureAEnabled()
    {
        yggdrasilEngine.IsEnabled("Feature.A", new Context());
    }
}

public class Program
{
    public static void Main(string[] args)
    {
        var summary = BenchmarkRunner.Run<YggBench>();
    }
}
