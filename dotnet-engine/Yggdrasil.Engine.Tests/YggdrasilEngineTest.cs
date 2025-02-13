using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using NUnit.Framework;
using System;
using Newtonsoft.Json.Linq;
using Yggdrasil;
using Yggdrasil.Test;
using System.Text.Json.Serialization;

public class TestVariantReadModel
{
    public TestVariantReadModel(string name, Payload? payload, bool enabled, bool featureEnabled)
    {
        Name = name;
        Payload = payload;
        Enabled = enabled;
        FeatureEnabled = featureEnabled;
    }

    public string Name { get; set; }
    public Payload? Payload { get; set; }
    public bool Enabled { get; set; }

    [JsonPropertyName("feature_enabled")]
    public bool FeatureEnabled { get; private set; }
}

public class Tests
{
    private JsonSerializerOptions options = new JsonSerializerOptions
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    //[Test]
    public void MassTestMemoryUsage()
    {
        // Arrange
        var basePath = Path.Combine("..", "..", "..", "..", "..", "client-specification", "specifications");
        var suitePath = Path.Combine(basePath, "01-simple-examples.json");
        var suiteData = JObject.Parse(File.ReadAllText(suitePath));

        var yggdrasilEngine = new YggdrasilEngine();

        var runTestFor = (Action lambda, string process) =>
        {

            // Baseline / warm up
            for (var i = 0; i < 1000000; i++)
            {
                lambda();
            }
            GC.Collect();

            var baseline = GC.GetTotalMemory(true);

            // Act
            for (var i = 0; i < 1000000; i++)
            {
                lambda();
            }
            GC.Collect();

            var memoryTotal = GC.GetTotalMemory(true);

            // Assert
            var diff = memoryTotal - baseline;
            Assert.LessOrEqual(diff, 200000, process + " has a potential memory leak. Diff: " + diff + " bytes");
        };

        runTestFor(() => yggdrasilEngine.IsEnabled("Feature.A", new Context()), "IsEnabled");
        runTestFor(() => yggdrasilEngine.GetVariant("Feature.A", new Context()), "GetVariant");
        runTestFor(() => yggdrasilEngine.TakeState(suiteData["state"].ToString()), "TakeState");
        runTestFor(() => yggdrasilEngine.GetMetrics(), "GetMetrics");
    }

    [Test]
    public void TestClientSpec()
    {
        var yggdrasilEngine = new YggdrasilEngine();
        var basePath = Path.Combine("..", "..", "..", "..", "..", "client-specification", "specifications");
        var indexFilePath = Path.Combine(basePath, "index.json");
        var testSuites = JArray.Parse(File.ReadAllText(indexFilePath));

        foreach (var suite in testSuites)
        {
            var suitePath = Path.Combine(basePath, suite.ToString());
            var suiteData = JObject.Parse(File.ReadAllText(suitePath));

            yggdrasilEngine.TakeState(suiteData["state"].ToString());

            var tests = suiteData["tests"] ?? new JArray();

            foreach (var test in tests)
            {

                var contextJson = test["context"].ToString();
                var context = JsonSerializer.Deserialize<Context>(contextJson, options) ?? new Context();
                var toggleName = (string)test["toggleName"];
                var expectedResult = (bool)test["expectedResult"];

                var (enabled, _) = yggdrasilEngine.IsEnabled(toggleName, context);
                if (enabled == null)
                {
                    enabled = false;
                }

                Assert.AreEqual(expectedResult, enabled, message: $"Failed client specification '{suite}': Failed test '{test["description"]}': expected {expectedResult}, got {enabled}");
            }

            var variantTests = suiteData["variantTests"] ?? new JArray();
            foreach (var test in variantTests)
            {
                var contextJson = test["context"].ToString();
                var context = JsonSerializer.Deserialize<Context>(contextJson, options) ?? new Context();
                var toggleName = (string)test["toggleName"];
                // Silly hack to deal with the legacy "feature_enabled" property on the specs
                var expectedResult = JsonSerializer.Deserialize<TestVariantReadModel>(test["expectedResult"].ToString(), options);
                var (variant, impressionData) = yggdrasilEngine.GetVariant(toggleName, context);
                if (variant == null)
                {
                    var (enabled, _) = yggdrasilEngine.IsEnabled(toggleName, context);
                    variant = new Variant("disabled", null, false, enabled ?? false);
                }

                Assert.AreEqual(expectedResult!.Name, variant.Name, message: $"Failed client specification '{suite}': Failed test '{test["description"]}': expected {expectedResult.Name}, got {variant.Name}");
                Assert.AreEqual(expectedResult!.Enabled, variant.Enabled, message: $"Failed client specification '{suite}': Failed test '{test["description"]}': expected {expectedResult.Enabled}, got {variant.Enabled}");
                Assert.AreEqual(expectedResult!.FeatureEnabled, variant.FeatureEnabled, message: $"Failed client specification '{suite}': Failed test '{test["description"]}': expected {expectedResult.FeatureEnabled}, got {variant.FeatureEnabled}");
                Assert.AreEqual(expectedResult!.Payload?.Type, variant.Payload?.Type, message: $"Failed client specification '{suite}': Failed test '{test["description"]}': expected {expectedResult.Payload?.Type}, got {variant.Payload?.Type}");
                Assert.AreEqual(expectedResult!.Payload?.Value, variant.Payload?.Value, message: $"Failed client specification '{suite}': Failed test '{test["description"]}': expected {expectedResult.Payload?.Value}, got {variant.Payload?.Value}");
            }

            Console.WriteLine($"Passed client specification {suite}");
        }
    }

    [Test]
    public void Simple_Feature_Is_Enabled()
    {
        var yggdrasilEngine = new YggdrasilEngine();
        var filePath = Path.Combine("..", "..", "..", "..", "..", "test-data", "simple.json");
        var json = File.ReadAllText(filePath);
        yggdrasilEngine.TakeState(json);
        var context = new Context();
        var (result, _) = yggdrasilEngine.IsEnabled("Feature.A", context);
        Assert.AreEqual(true, result);
    }

    [Test]
    public void Custom_Strategies_Required_But_Not_Configured_Returns_False()
    {
        var yggdrasilEngine = new YggdrasilEngine();
        var filePath = Path.Combine("..", "..", "..", "..", "..", "test-data", "simple.json");
        var json = File.ReadAllText(filePath);
        yggdrasilEngine.TakeState(json);
        var context = new Context();
        var (result, _) = yggdrasilEngine.IsEnabled("Feature.D", context);
        Assert.AreEqual(false, result);
    }

    [Test]
    public void Custom_Strategies_Required_And_Configured_Succeeds()
    {
        var yggdrasilEngine = new YggdrasilEngine(new List<IStrategy>
        {
            new CustomStrategyReturningTrue("custom"),
            new CustomStrategyReturningTrue("cus-tom")
        });

        var filePath = Path.Combine("..", "..", "..", "..", "..", "test-data", "simple.json");
        var json = File.ReadAllText(filePath);
        yggdrasilEngine.TakeState(json);
        var context = new Context();
        var (result, _) = yggdrasilEngine.IsEnabled("Feature.A", context);
        Assert.AreEqual(true, result);
    }

    [Test]
    public void Custom_Strategies_Correct_Names_Despite_Ordering()
    {
        var yggdrasilEngine = new YggdrasilEngine(new List<IStrategy>
        {
            new CustomStrategyReturningTrue("custom"),
            new CustomStrategyReturningTrue("cus-tom")
        });

        var filePath = Path.Combine("..", "..", "..", "..", "..", "test-data", "simple.json");
        var json = File.ReadAllText(filePath);
        yggdrasilEngine.TakeState(json);
        var context = new Context();
        var (result, impressionData) = yggdrasilEngine.IsEnabled("Feature.E", context);
        Assert.AreEqual(true, result);
    }

    [Test]
    public void Impression_Data_Test_Enabled()
    {
        var testDataObject = new
        {
            Version = 2,
            Features = new[] {
                new {
                    Name = "with.impression.data",
                    Type = "release",
                    Enabled = true,
                    ImpressionData = true,
                    Strategies = new [] {
                        new {
                            Name = "default",
                            Parameters = new Dictionary<string, string>()
                        }
                    }
                }
            }
        };

        var testData = JsonSerializer.Serialize(testDataObject, options);
        var engine = new YggdrasilEngine();
        engine.TakeState(testData);
        var featureName = "with.impression.data";
        var (result, impressionData) = engine.IsEnabled(featureName, new Context());
        var shouldEmit = engine.ShouldEmitImpressionEvent(featureName);
        Assert.NotNull(result);
        Assert.IsTrue(result);
        Assert.NotNull(shouldEmit);
        Assert.IsTrue(shouldEmit);
    }

    [Test]
    public void Impression_Data_Test_Disabled()
    {
        var testDataObject = new
        {
            Version = 2,
            Features = new[] {
                new {
                    Name = "with.impression.data.false",
                    Type = "release",
                    Enabled = true,
                    ImpressionData = false,
                    Strategies = new [] {
                        new {
                            Name = "default",
                            Parameters = new Dictionary<string, string>()
                        }
                    }
                }
            }
        };

        var testData = JsonSerializer.Serialize(testDataObject, options);
        var engine = new YggdrasilEngine();
        engine.TakeState(testData);
        var featureName = "with.impression.data.false";
        var (result, impressionData) = engine.IsEnabled(featureName, new Context());
        var shouldEmit = engine.ShouldEmitImpressionEvent(featureName);
        Assert.NotNull(result);
        Assert.IsTrue(result);
        Assert.IsFalse(shouldEmit);
    }

    [Test]
    public void Invalid_Json_Raises_An_Error()
    {
        var testData = "{\"weCloseBraces\": false";
        var engine = new YggdrasilEngine();
        Assert.Throws<YggdrasilEngineException>(() => engine.TakeState(testData));
    }

    [Test]
    public void Valid_Json_With_An_Invalid_State_Update_Raises_An_Error()
    {
        var testData = "{\"weCloseBraces\": true}";
        var engine = new YggdrasilEngine();
        Assert.Throws<YggdrasilEngineException>(() => engine.TakeState(testData));
    }

    [Test]
    public void Features_That_Get_Set_Can_Be_Listed()
    {
        var testDataObject = new
        {
            Version = 2,
            Features = new[] {
                new {
                    Name = "with.impression.data",
                    Type = "release",
                    Enabled = true,
                    ImpressionData = true,
                    Strategies = new [] {
                        new {
                            Name = "default",
                            Parameters = new Dictionary<string, string>()
                        }
                    }
                }
            }
        };

        var testData = JsonSerializer.Serialize(testDataObject, options);
        var engine = new YggdrasilEngine();
        engine.TakeState(testData);
        var knownFeatures = engine.ListKnownToggles();
        Assert.AreEqual(1, knownFeatures.Count);
    }
}
