using System.Text.Json.Serialization;

namespace Yggdrasil;

public sealed class Context
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

public sealed class Variant
{
    public string Name { get; set; } = null!;
    public Payload? Payload { get; set; }
    public bool Enabled { get; set; } = false;
}

public sealed class Payload
{
    public string PayloadType { get; set; } = null!;
    public string Value { get; set; } = null!;

    public override bool Equals(object? obj)
    {
        if (obj == null)
            return false;

        var payload = (Payload)obj;
        return Value == payload.Value && PayloadType == payload.PayloadType;
    }

    public override int GetHashCode()
    {
        unchecked
        {
            int hash = 17;
            hash = hash * 23 + (Value?.GetHashCode() ?? 0);
            hash = hash * 23 + (PayloadType?.GetHashCode() ?? 0);
            return hash;
        }
    }
}

public class YggdrasilEngineException : Exception
{
    public YggdrasilEngineException(string message)
        : base(message) { }
}

public sealed class FeatureCount
{
    public long Yes { get; set; } = 0;
    public long No { get; set; } = 0;
    public Dictionary<string, long> Variants { get; set; } = new Dictionary<string, long>();
}

public sealed class MetricsBucket
{
    public Dictionary<string, FeatureCount> Toggles { get; set; } = null!;
    public DateTimeOffset Start { get; set; } = DateTimeOffset.MinValue;
    public DateTimeOffset Stop { get; set; } = DateTimeOffset.MinValue;
}
