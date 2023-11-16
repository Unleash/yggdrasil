using System.Text.Json;

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
    var knownStrategies = knownStrategiesPtr == IntPtr.Zero
      ? null
      : FFIReader.ReadComplex<string[]>(knownStrategiesPtr);

    customStrategies = new CustomStrategies(knownStrategies);

    if (strategies != null)
    {
      customStrategies.RegisterCustomStrategies(strategies);
    }
  }

  public bool ShouldEmitImpressionEvent(string featureName)
  {
    var shouldEmitImpressionEventPtr = FFI.ShouldEmitImpressionEvent(state, featureName);

    var shouldEmitImpressionEvent = shouldEmitImpressionEventPtr == IntPtr.Zero
      ? null
      : FFIReader.ReadPrimitive<bool>(shouldEmitImpressionEventPtr);

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

    if (takeStatePtr == IntPtr.Zero)
    {
      return;
    }

    FFIReader.CheckResponse(takeStatePtr);

    customStrategies.MapFeatures(json);
  }

  public bool? IsEnabled(string toggleName, Context context)
  {
    var customStrategyPayload = customStrategies.GetCustomStrategyPayload(toggleName, context);
    string contextJson = JsonSerializer.Serialize(context, options);
    var isEnabledPtr = FFI.CheckEnabled(state, toggleName, contextJson, customStrategyPayload);

    var isEnabled = isEnabledPtr == IntPtr.Zero
      ? null
      : FFIReader.ReadPrimitive<bool>(isEnabledPtr);

    return isEnabled ?? false;
  }

  public Variant? GetVariant(string toggleName, Context context)
  {
    var customStrategyPayload = customStrategies.GetCustomStrategyPayload(toggleName, context);
    var contextJson = JsonSerializer.Serialize(context, options);
    var variantPtr = FFI.CheckVariant(state, toggleName, contextJson, customStrategyPayload);

    var variant = variantPtr == IntPtr.Zero
      ? null
      : FFIReader.ReadComplex<Variant>(variantPtr);

    return variant;
  }

  public MetricsBucket? GetMetrics()
  {
    var metricsPtr = FFI.GetMetrics(state);

    var metrics = metricsPtr == IntPtr.Zero
      ? null
      : FFIReader.ReadComplex<MetricsBucket>(metricsPtr);

    return metrics;
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
