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

public class EngineResponse
{
    [JsonPropertyName("error_message")]
    public string? ErrorMessage { get; set; }

    [JsonPropertyName("status_code")]
    public string? StatusCode { get; set; }
}

public class EngineResponse<TValue> : EngineResponse
{
    public TValue? Value { get; set; }
}

public class Variant
{
    public Variant(string name, Payload? payload, bool enabled, bool feature_enabled)
    {
        Name = name;
        Payload = payload;
        Enabled = enabled;
        Feature_Enabled = feature_enabled;
    }

    public static readonly Variant DISABLED_VARIANT = new Variant("disabled", null, false, false);

    public string Name { get; set; }
    public Payload? Payload { get; set; }
    [JsonPropertyName("enabled")]
    public bool Enabled { get; set; }
    [JsonPropertyName("feature_enabled")]
    public bool Feature_Enabled { get; set; }
}

public class Payload
{
    public Payload(string type, string value)
    {
        Type = type;
        Value = value;
    }

    public string Type { get; set; }
    public string Value { get; set; }

    public override bool Equals(object obj)
    {
        if (obj == this) return true;
        if (obj == null) return false;

        var payload = (Payload)obj;

        return Equals(payload.Type, Type) && Equals(payload.Value, Value);
    }

    public override int GetHashCode()
    {
        return new { Type, Value }.GetHashCode();
    }
}

public class YggdrasilEngineException : Exception
{
    public YggdrasilEngineException(string message) : base(message) { }
}

public class FeatureCount
{
    public FeatureCount(long yes, long no, Dictionary<string, long> variants)
    {
        Yes = yes;
        No = no;
        Variants = variants;
    }

    public long Yes { get; set; }
    public long No { get; set; }
    public Dictionary<string, long> Variants { get; set; }
}


public class MetricsBucket
{
    public MetricsBucket(Dictionary<string, FeatureCount> toggles, DateTimeOffset start, DateTimeOffset stop)
    {
        Toggles = toggles;
        Start = start;
        Stop = stop;
    }

    public Dictionary<string, FeatureCount> Toggles { get; set; }

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
        return Strategy.IsEnabled(Parameters, context);
    }
}
