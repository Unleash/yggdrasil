﻿using System.Text.Json;

namespace Yggdrasil;

public class YggdrasilEngine
{
    private CustomStrategies customStrategies;

    private JsonSerializerOptions options = new JsonSerializerOptions
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    private IntPtr state;

    public YggdrasilEngine(List<IStrategy>? strategies = null)
    {
        state = FFI.NewEngine();

        var knownStrategiesPtr = FFI.BuiltInStrategies(state);
        var knownStrategies = FFIReader.ReadResponse<string[]>(knownStrategiesPtr);

        customStrategies = new CustomStrategies(knownStrategies);

        if (strategies != null)
        {
            customStrategies.RegisterCustomStrategies(strategies);
        }
    }

    public bool ShouldEmitImpressionEvent(string featureName)
    {
        var shouldEmitImpressionEventPtr = FFI.ShouldEmitImpressionEvent(state, featureName);
        var shouldEmitImpressionEvent = FFIReader.ReadPrimitive<bool>(shouldEmitImpressionEventPtr);

        return shouldEmitImpressionEvent ?? false;
    }

    public void Dispose()
    {
        FFI.FreeEngine(this.state);
        GC.SuppressFinalize(this);
    }

    public void TakeState(string json)
    {
        var takeStatePtr = FFI.TakeState(state, json);
        FFIReader.CheckResponse(takeStatePtr);

        customStrategies.MapFeatures(json);
    }

    public bool? IsEnabled(string toggleName, Context context)
    {
        var customStrategyPayload = customStrategies.GetCustomStrategyPayload(toggleName, context);
        string contextJson = JsonSerializer.Serialize(context, options);
        var isEnabledPtr = FFI.CheckEnabled(state, toggleName, contextJson, customStrategyPayload);

        return FFIReader.ReadPrimitive<bool>(isEnabledPtr);
    }

    public Variant? GetVariant(string toggleName, Context context)
    {
        var customStrategyPayload = customStrategies.GetCustomStrategyPayload(toggleName, context);
        var contextJson = JsonSerializer.Serialize(context, options);
        var variantPtr = FFI.CheckVariant(state, toggleName, contextJson, customStrategyPayload);

        return FFIReader.ReadComplex<Variant>(variantPtr);
    }

    public MetricsBucket? GetMetrics()
    {
        var metricsPtr = FFI.GetMetrics(state);
        return FFIReader.ReadComplex<MetricsBucket>(metricsPtr);
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
