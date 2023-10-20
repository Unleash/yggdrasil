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

public class EngineResponse<TResult> : EngineResponse {
    public TResult? Value { get; set; }
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
}
