using System.Text.Json;
using static Yggdrasil.FFI;

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

    public QuickCheckResult IsEnabled(string toggleName, Context context)
    {
        return FFI.QuickCheck(state, toggleName, context, customStrategies.GetCustomStrategies(toggleName, context));
        // return true;
        // var customStrategyPayload = customStrategies.GetCustomStrategyPayload(toggleName, context);
        // string contextJson = JsonSerializer.Serialize(context, options);
        // var isEnabledPtr = FFI.CheckEnabled(state, toggleName, contextJson, customStrategyPayload);

        // return true;
        // return FFIReader.ReadPrimitive<bool>(isEnabledPtr);
    }

    public QuickVariantResult GetVariant(string toggleName, Context context)
    {
        return FFI.QuickVariant(state, toggleName, context, customStrategies.GetCustomStrategies(toggleName, context));

        // var customStrategyPayload = customStrategies.GetCustomStrategyPayload(toggleName, context);
        // var contextJson = JsonSerializer.Serialize(context, options);
        // var variantPtr = FFI.CheckVariant(state, toggleName, contextJson, customStrategyPayload);

        // return FFIReader.ReadComplex<Variant>(variantPtr);
    }

    public MetricsBucket? GetMetrics()
    {
        var metricsPtr = FFI.GetMetrics(state);
        return FFIReader.ReadComplex<MetricsBucket>(metricsPtr);
    }

    public ICollection<FeatureDefinition> ListKnownToggles()
    {
        var featureDefinitionsPtr = FFI.ListKnownToggles(state);
        var knownFeatures = FFIReader.ReadComplex<List<FeatureDefinition>>(featureDefinitionsPtr);
        if (knownFeatures == null)
        {
            return new List<FeatureDefinition>();
        }
        return knownFeatures;
    }
}
