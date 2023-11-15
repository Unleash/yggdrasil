using System.Runtime.InteropServices;
using System.Text.Json;

namespace Yggdrasil;

public class YggdrasilEngine
{
    private CustomStrategies customStrategies = new CustomStrategies();

    private JsonSerializerOptions options = new JsonSerializerOptions
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    private IntPtr state;

    public YggdrasilEngine(List<IStrategy>? strategies = null)
    {
        state = FFI.NewEngine();

        if (strategies != null)
        {
            customStrategies.RegisterCustomStrategies(strategies);
        }
    }

    public bool ShouldEmitImpressionEvent(string featureName)
    {
        var shouldEmitImpressionEventPtr = FFI.ShouldEmitImpressionEvent(state, featureName);
        if (shouldEmitImpressionEventPtr == IntPtr.Zero)
        {
            return false;
        }

        var shouldEmitImpressionEventJson = Marshal.PtrToStringUTF8(shouldEmitImpressionEventPtr);

        FFI.FreeResponse(shouldEmitImpressionEventPtr);

        var shouldEmitImpressionEventResult =
            shouldEmitImpressionEventJson != null
                ? JsonSerializer.Deserialize<EngineResponse<bool>>(shouldEmitImpressionEventJson, options)
                : null;
        
        if (shouldEmitImpressionEventResult?.StatusCode == "Error")
        {
            throw new YggdrasilEngineException($"Error: {shouldEmitImpressionEventResult?.ErrorMessage}");
        }

        return shouldEmitImpressionEventResult?.Value ?? false;
    }

    public void Dispose()
    {
        FFI.FreeEngine(this.state);
        GC.SuppressFinalize(this);
    }

    public void TakeState(string json)
    {
        var takeStatePtr = FFI.TakeState(state, json);

        if (takeStatePtr == IntPtr.Zero)
        {
            return;
        }

        var takeStateJson = Marshal.PtrToStringUTF8(takeStatePtr);

        FFI.FreeResponse(takeStatePtr);

        var takeStateResult =
            takeStateJson != null
                ? JsonSerializer.Deserialize<EngineResponse>(takeStateJson, options)
                : null;

        if (takeStateResult?.StatusCode == "Error")
        {
            throw new YggdrasilEngineException($"Error: {takeStateResult?.ErrorMessage}");
        }

        customStrategies.MapFeatures(json);
    }

    public bool? IsEnabled(string toggleName, Context context)
    {
        var customStrategyPayload = customStrategies.GetCustomStrategyPayload(toggleName, context);
        string contextJson = JsonSerializer.Serialize(context, options);
        var isEnabledPtr = FFI.CheckEnabled(state, toggleName, contextJson, customStrategyPayload);

        if (isEnabledPtr == IntPtr.Zero)
        {
            return false;
        }

        var isEnabledJson = Marshal.PtrToStringUTF8(isEnabledPtr);

        FFI.FreeResponse(isEnabledPtr);

        var isEnabledResult =
            isEnabledJson != null
                ? JsonSerializer.Deserialize<EngineResponse<bool?>>(isEnabledJson, options)
                : null;

        if (isEnabledResult?.StatusCode == "Error")
        {
            throw new YggdrasilEngineException($"Error: {isEnabledResult?.ErrorMessage}");
        }

        return isEnabledResult?.Value;
    }

    public Variant? GetVariant(string toggleName, Context context)
    {
        var customStrategyPayload = customStrategies.GetCustomStrategyPayload(toggleName, context);
        var contextJson = JsonSerializer.Serialize(context, options);
        var variantPtr = FFI.CheckVariant(state, toggleName, contextJson, customStrategyPayload);

        if (variantPtr == IntPtr.Zero)
        {
            return null;
        }

        var variantJson = Marshal.PtrToStringUTF8(variantPtr);

        FFI.FreeResponse(variantPtr);

        var variantResult =
            variantJson != null
                ? JsonSerializer.Deserialize<EngineResponse<Variant>>(variantJson, options)
                : null;

        if (variantResult?.StatusCode == "Error")
        {
            throw new YggdrasilEngineException($"Error: {variantResult?.ErrorMessage}");
        }

        return variantResult?.Value;
    }

    public MetricsBucket? GetMetrics()
    {
        var metricsPtr = FFI.GetMetrics(state);

        if (metricsPtr == IntPtr.Zero)
        {
            return null;
        }

        var metricsJson = Marshal.PtrToStringUTF8(metricsPtr);

        FFI.FreeResponse(metricsPtr);

        var metricsResult =
            metricsJson != null
                ? JsonSerializer.Deserialize<EngineResponse<MetricsBucket>>(metricsJson, options)
                : null;

        if (metricsResult?.StatusCode == "Error")
        {
            throw new YggdrasilEngineException($"Error: {metricsResult?.ErrorMessage}");
        }

        return metricsResult?.Value;
    }

    public void CountFeature(string featureName, bool enabled)
    {
        FFI.CountToggle(state, featureName, enabled);
    }

    public void CountVariant(string featureName, string variantName)
    {
        FFI.CountVariant(state, featureName, variantName);
    }
}
