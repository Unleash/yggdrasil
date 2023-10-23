
using System.Runtime.InteropServices;
using System.Text.Json;


namespace Unleash;

public class UnleashEngine
{
    private JsonSerializerOptions options = new JsonSerializerOptions
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    private IFFIAccess platformEngine;

    private static IFFIAccess GetPlatformEngine() { 
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux)) {
            return new FFILinux();
        } else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX)) {
            return new FFIMacOS();
        } else if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows)) {
            return new FFIWin();
        } else {
            throw new PlatformNotSupportedException();
        }
    }

    private IntPtr state;

    public UnleashEngine()
    {
        platformEngine = GetPlatformEngine();
        state = platformEngine.NewEngine();
    }

    public void Dispose()
    {
        platformEngine.FreeEngine(this.state);
        GC.SuppressFinalize(this);
    }

    public void TakeState(string json)
    {
        var takeStatePtr = platformEngine.TakeState(state, json);

        if (takeStatePtr == IntPtr.Zero)
        {
            return;
        }

        var takeStateJson = Marshal.PtrToStringUTF8(takeStatePtr);

        platformEngine.FreeResponse(takeStatePtr);

        var takeStateResult = takeStateJson != null ?
            JsonSerializer.Deserialize<EngineResponse>(takeStateJson, options) :
            null;

        if (takeStateResult?.StatusCode == "Error") {
            throw new UnleashException($"Error: {takeStateResult?.ErrorMessage}");
        }
    }

    public bool IsEnabled(string toggleName, Context context)
    {
        string contextJson = JsonSerializer.Serialize(context, options);
        var isEnabledPtr = platformEngine.CheckEnabled(state, toggleName, contextJson);

        if (isEnabledPtr == IntPtr.Zero)
        {
            return false;
        }

        var isEnabledJson = Marshal.PtrToStringUTF8(isEnabledPtr);

        platformEngine.FreeResponse(isEnabledPtr);

        var isEnabledResult = isEnabledJson != null ?
            JsonSerializer.Deserialize<EngineResponse<bool?>>(isEnabledJson, options) :
            null;


        if (isEnabledResult?.StatusCode == "Error") {
            throw new UnleashException($"Error: {isEnabledResult?.ErrorMessage}");
        }

        return isEnabledResult?.Value ?? false;
    }

    public Variant? GetVariant(string toggleName, Context context)
    {
        var contextJson = JsonSerializer.Serialize(context, options);
        var variantPtr = platformEngine.CheckVariant(state, toggleName, contextJson);

        if (variantPtr == IntPtr.Zero)
        {
            return null;
        }

        var variantJson = Marshal.PtrToStringUTF8(variantPtr);

        platformEngine.FreeResponse(variantPtr);

        var variantResult = variantJson != null ?
            JsonSerializer.Deserialize<EngineResponse<Variant>>(variantJson, options) :
            null;

        if (variantResult?.StatusCode == "Error") {
            throw new UnleashException($"Error: {variantResult?.ErrorMessage}");
        }

        return variantResult?.Value ?? new Variant() { Enabled = false, Name = "disabled", Payload = null };
    }

    public Dictionary<string, int>? GetMetrics() {
        var metricsPtr = platformEngine.GetMetrics(state);

        if (metricsPtr == IntPtr.Zero)
        {
            return null;
        }

        var metricsJson = Marshal.PtrToStringUTF8(metricsPtr);

        platformEngine.FreeResponse(metricsPtr);

        var metricsResult = metricsJson != null ?
            JsonSerializer.Deserialize<EngineResponse<Dictionary<string, int>>>(metricsJson, options) :
            null;

        if (metricsResult?.StatusCode == "Error") {
            throw new UnleashException($"Error: {metricsResult?.ErrorMessage}");
        }

        return metricsResult?.Value;
    }

    public void CountToggle(string toggle, bool enabled)
    {
        platformEngine.CountToggle(state, toggle, enabled);
    }

    public void CountVariant(string toggle, string variant)
    {
        platformEngine.CountVariant(state, toggle, variant);
    }
}