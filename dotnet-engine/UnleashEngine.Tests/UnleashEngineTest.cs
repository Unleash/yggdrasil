using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using NUnit.Framework;
using Unleash;
using System;
using Newtonsoft.Json.Linq;


public class Tests
{
    private JsonSerializerOptions options = new JsonSerializerOptions
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    [Test] 
    public void MassTestMemoryUsage() {
        // Arrange
        var basePath = Path.Combine("..", "..", "..", "..", "..", "..", "client-specification", "specifications");
        var suitePath = Path.Combine(basePath, "01-simple-examples.json");
        var suiteData = JObject.Parse(File.ReadAllText(suitePath));

        var unleashEngine = new UnleashEngine();
        //unleashEngine.TakeState(suiteData["state"].ToString());

        var runTestFor = (Action lambda, string process) => {

            // Baseline / warm up
            for (var i = 0; i < 1000000; i++) {
                lambda();
            }
            GC.Collect();

            var baseline = GC.GetTotalMemory(true);

            // Act
            for (var i = 0; i < 1000000; i++) {
                lambda();
            }
            GC.Collect();

            var memoryTotal = GC.GetTotalMemory(true);

            // Assert
            var diff = memoryTotal - baseline;
            Assert.LessOrEqual(diff, 200000, process + " has a potential memory leak. Diff: " + diff + " bytes");
        };

        runTestFor(() => unleashEngine.IsEnabled("Feature.A", new Context()), "IsEnabled");
        runTestFor(() => unleashEngine.GetVariant("Feature.A", new Context()), "GetVariant");
        runTestFor(() => unleashEngine.TakeState(suiteData["state"].ToString()), "TakeState");
        runTestFor(() => unleashEngine.GetMetrics(), "GetMetrics");
    }

    [Test]
    public void TestClientSpec()
    {
        var unleashEngine = new UnleashEngine();
        var basePath = Path.Combine("..", "..", "..", "..", "..", "..", "client-specification", "specifications");
        var indexFilePath = Path.Combine(basePath, "index.json");
        var testSuites = JArray.Parse(File.ReadAllText(indexFilePath));

        foreach (var suite in testSuites)
        {
            var suitePath = Path.Combine(basePath, suite.ToString());
            var suiteData = JObject.Parse(File.ReadAllText(suitePath));

            unleashEngine.TakeState(suiteData["state"].ToString());

            var tests = suiteData["tests"] ?? new JArray();
            foreach (var test in tests)
            {

                var contextJson = test["context"].ToString();
                var context = JsonSerializer.Deserialize<Context>(contextJson, options);
                var toggleName = (string)test["toggleName"];
                var expectedResult = (bool)test["expectedResult"];

                var result = unleashEngine.IsEnabled(toggleName, context);

                Assert.AreEqual(expectedResult, result, message: $"Failed client specification '{suite}': Failed test '{test["description"]}': expected {expectedResult}, got {result}");
            }

            var variantTests = suiteData["variantTests"] ?? new JArray();
            foreach (var test in variantTests)
            {
                var contextJson = test["context"].ToString();
                var context = JsonSerializer.Deserialize<Context>(contextJson, options);
                var toggleName = (string)test["toggleName"];
                // Silly hack to apply formatting to the string from the spec
                var expectedResult = JsonSerializer.Serialize(JsonSerializer.Deserialize<Variant>(test["expectedResult"].ToString(), options), options);

                var result = unleashEngine.GetVariant(toggleName, context);
                var jsonResult = JsonSerializer.Serialize(result, options);

                Assert.AreEqual(expectedResult, jsonResult, message: $"Failed client specification '{suite}': Failed test '{test["description"]}': expected {expectedResult}, got {result}");
            }

            Console.WriteLine($"Passed client specification {suite}");
        }
    }

}