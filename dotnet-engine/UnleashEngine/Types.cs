using System.Text.Json.Serialization;

namespace Unleash;

public class Context
{
    public string? UserId { get; set; }
    public string? SessionId { get; set; }
    public string? RemoteAddress { get; set; }
    public string? Environment { get; set; }
    public string? AppName { get; set; }
    public string? CurrentTime { get; set; }
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
    public string Name { get; set; }
    public Payload? Payload { get; set; }
    public bool Enabled { get; set; }
}

public class Payload
{
    public string? PayloadType { get; set; }
    public string? Value { get; set; }
}

public class UnleashException : Exception
{
    public UnleashException(string message) : base(message) { }
}

public class FeatureCount
{
    public long Yes { get; set; }
    public long No { get; set; }
    public Dictionary<string, long> Variants { get; set; }
}


public class MetricsBucket
{
    public Dictionary<string, FeatureCount> Toggles { get; set; }

    public DateTimeOffset Start { get; set; }
    public DateTimeOffset Stop { get; set; }
}
