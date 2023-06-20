
using System.Runtime.InteropServices;
using System.Text.Json;


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


public class UnleashEngine
{
    private JsonSerializerOptions options = new JsonSerializerOptions
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    private IntPtr state;
    private const string DLL_PATH = "../../../../../target/release/libyggdrasilffi.so";

    [DllImport(DLL_PATH)]
    private static extern IntPtr engine_new();

    [DllImport(DLL_PATH)]
    private static extern void engine_free(IntPtr ptr);

    [DllImport(DLL_PATH)]
    private static extern IntPtr engine_take_state(IntPtr ptr, string json);

    [DllImport(DLL_PATH)]
    private static extern bool engine_is_enabled(IntPtr ptr, string toggle_name, string context);

    [DllImport(DLL_PATH)]
    private static extern IntPtr engine_get_variant(IntPtr ptr, string toggle_name, string context);

    [DllImport(DLL_PATH)]
    private static extern void engine_free_variant_def(IntPtr ptr);

    public UnleashEngine()
    {
        this.state = engine_new();
    }

    public void Dispose()
    {
        engine_free(this.state);
        GC.SuppressFinalize(this);
    }

    public void TakeState(string json)
    {
        engine_take_state(state, json);
    }

    public bool IsEnabled(string toggleName, Context context)
    {
        string contextJson = JsonSerializer.Serialize(context, options);
        return engine_is_enabled(state, toggleName, contextJson);
    }

    public Variant? GetVariant(string toggleName, Context context)
    {
        var contextJson = JsonSerializer.Serialize(context, options);
        var variantPtr = engine_get_variant(state, toggleName, contextJson);

        if (variantPtr == IntPtr.Zero)
        {
            return null;
        }
        else
        {
            var variantJson = Marshal.PtrToStringUTF8(variantPtr);

            var variant = JsonSerializer.Deserialize<Variant>(variantJson, options);

            engine_free_variant_def(variantPtr);

            return variant;
        }
    }
}