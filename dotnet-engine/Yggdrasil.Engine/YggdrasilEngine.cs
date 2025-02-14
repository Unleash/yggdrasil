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

    public EnabledResult IsEnabled(string toggleName, Context context)
    {
        return FFI.IsEnabled(state, toggleName, context, customStrategies.GetCustomStrategies(toggleName, context));
    }

    public VariantResult GetVariant(string toggleName, Context context, string defaultVariantName)
    {
        return FFI.GetVariant(state, toggleName, context, defaultVariantName, customStrategies.GetCustomStrategies(toggleName, context));
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
