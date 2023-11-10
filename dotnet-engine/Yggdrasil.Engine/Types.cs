using System.Text.Json.Serialization;

namespace Yggdrasil;

public class Context
{
    public string? UserId { get; set; }
    public string? SessionId { get; set; }
    public string? RemoteAddress { get; set; }
    public string? Environment { get; set; }
    public string? AppName { get; set; }
    public DateTimeOffset? CurrentTime { get; set; }
    public Dictionary<string, string>? Properties { get; set; }
}

public class EngineResponse {
    [JsonPropertyName("error_message")]
    public string? ErrorMessage { get; set; }

    [JsonPropertyName("status_code")]
    public string? StatusCode { get; set; }
}

public class EngineResponse<TValue> : EngineResponse {
    public TValue? Value { get; set; }
}

public class Variant
{
    public string? Name { get; set; }
    public Payload? Payload { get; set; }
    public bool Enabled { get; set; }
}

public class Payload
{
    public string? PayloadType { get; set; }
    public string? Value { get; set; }

    public override bool Equals(object? obj)
    {
        if (obj == null)
            return false;

        var payload = (Payload)obj;
        return Value == payload.Value && PayloadType == payload.PayloadType;
    }

    public override int GetHashCode()
    {
        return HashCode.Combine(Value, PayloadType);
    }
}

public class YggdrasilEngineException : Exception
{
    public YggdrasilEngineException(string message) : base(message) { }
}

public class FeatureCount
{
    public long Yes { get; set; }
    public long No { get; set; }
    public Dictionary<string, long>? Variants { get; set; }
}


public class MetricsBucket
{
    public Dictionary<string, FeatureCount>? Toggles { get; set; }

    public DateTimeOffset Start { get; set; }
    public DateTimeOffset Stop { get; set; }
}

/// <summary>
/// Defines a strategy for enabling a feature.
/// </summary>
public interface IStrategy
{
    /// <summary>
    /// Gets the stragegy name 
    /// </summary>
    string Name { get; }

    /// <summary>
    /// Calculates if the strategy is enabled for a given context
    /// </summary>
    bool IsEnabled(Dictionary<string, string> parameters, Context context);
}

class StrategyDefinition
{
    public string Name { get; set; } = "";

    public Dictionary<string, string>? Parameters { get; set; }
}

class Feature
{
    public string Name { get; set; } = "";
    public List<StrategyDefinition>? Strategies { get; set; }
}

class FeatureCollection
{
    public List<Feature>? Features { get; set; }
}

class MappedFeature
{
    public MappedFeature(Feature feature, List<MappedStrategy> strategies)
    {
        Name = feature.Name;
        Strategies = strategies;
    }

    public string Name { get; }
    public List<MappedStrategy> Strategies { get; }
}

class MappedStrategy
{
    public MappedStrategy(int index, string strategyName, Dictionary<string, string> parameters, IStrategy strategy)
    {
            ResultName = $"customStrategy{index + 1}";
            StrategyName = strategyName;
            Strategy = strategy;
            Parameters = parameters;
    }

    public string ResultName { get; private set; }

    public string StrategyName { get; private set; }

    public IStrategy Strategy { get; private set; }

    public Dictionary<string, string> Parameters { get; private set; }

    public bool IsEnabled(Context context)
    {
        return Strategy.IsEnabled(Parameters , context);
    }
}