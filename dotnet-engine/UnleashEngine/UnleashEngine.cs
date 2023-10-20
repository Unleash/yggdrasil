
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

        var takeStateResult = takeStateJson != null ?
            JsonSerializer.Deserialize<EngineResponse>(takeStateJson, options) :
            null;

        platformEngine.FreeResponse(takeStatePtr);

        if (takeStateResult?.StatusCode == "Error") {
            throw new Exception($"Error: {takeStateResult?.ErrorMessage}");
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
        else
        {
            var isEnabledJson = Marshal.PtrToStringUTF8(isEnabledPtr);

            var isEnabledResult = isEnabledJson != null ?
                JsonSerializer.Deserialize<EngineResponse<bool?>>(isEnabledJson, options) :
                null;

            platformEngine.FreeResponse(isEnabledPtr);


            if (isEnabledResult?.StatusCode == "Error") {
                throw new Exception($"Error: {isEnabledResult?.ErrorMessage}");
            }

            return isEnabledResult?.Value ?? false;
        }
    }

    public Variant? GetVariant(string toggleName, Context context)
    {
        var contextJson = JsonSerializer.Serialize(context, options);
        var variantPtr = platformEngine.CheckVariant(state, toggleName, contextJson);

        if (variantPtr == IntPtr.Zero)
        {
            return null;
        }
        else
        {
            var variantJson = Marshal.PtrToStringUTF8(variantPtr);

            var variantResult = variantJson != null ?
                JsonSerializer.Deserialize<EngineResponse<Variant>>(variantJson, options) :
                null;

            platformEngine.FreeResponse(variantPtr);

            if (variantResult?.StatusCode == "Error") {
                throw new Exception($"Error: {variantResult?.ErrorMessage}");
            }

            return variantResult?.Value ?? new Variant() { Enabled = false, Name = "disabled", Payload = null };
        }
    }

    public Dictionary<string, int>? GetMetrics() {
        var metricsPtr = platformEngine.GetMetrics(state);

        if (metricsPtr == IntPtr.Zero)
        {
            return null;
        }
        else
        {
            var metricsJson = Marshal.PtrToStringUTF8(metricsPtr);

            var metricsResult = metricsJson != null ?
                JsonSerializer.Deserialize<EngineResponse<Dictionary<string, int>>>(metricsJson, options) :
                null;

            platformEngine.FreeResponse(metricsPtr);

            if (metricsResult?.StatusCode == "Error") {
                throw new Exception($"Error: {metricsResult?.ErrorMessage}");
            }
            
            return metricsResult?.Value;
        }
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