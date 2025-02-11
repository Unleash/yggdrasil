using System;
using System.Runtime.InteropServices;

namespace Yggdrasil;

internal static class FFI
{
    private static IntPtr _libHandle;

    static FFI()
    {
        _libHandle = NativeLibLoader.LoadNativeLibrary();

        new_engine = Marshal.GetDelegateForFunctionPointer<NewEngineDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "new_engine"));
        free_engine = Marshal.GetDelegateForFunctionPointer<FreeEngineDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "free_engine"));
        get_metrics = Marshal.GetDelegateForFunctionPointer<GetMetricsDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "get_metrics"));
        take_state = Marshal.GetDelegateForFunctionPointer<TakeStateDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "take_state"));
        check_enabled = Marshal.GetDelegateForFunctionPointer<CheckEnabledDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "check_enabled"));
        check_variant = Marshal.GetDelegateForFunctionPointer<CheckVariantDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "check_variant"));
        free_response = Marshal.GetDelegateForFunctionPointer<FreeResponseDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "free_response"));
        count_toggle = Marshal.GetDelegateForFunctionPointer<CountToggleDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "count_toggle"));
        count_variant = Marshal.GetDelegateForFunctionPointer<CountVariantDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "count_variant"));
        should_emit_impression_event = Marshal.GetDelegateForFunctionPointer<ShouldEmitImpressionEventDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "should_emit_impression_event"));
        built_in_strategies = Marshal.GetDelegateForFunctionPointer<BuiltInStrategiesDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "built_in_strategies"));
        list_known_toggles = Marshal.GetDelegateForFunctionPointer<ListKnownTogglesDelegate>(NativeLibLoader.LoadFunctionPointer(_libHandle, "list_known_toggles"));
    }

    private delegate IntPtr NewEngineDelegate();
    private delegate void FreeEngineDelegate(IntPtr ptr);
    private delegate IntPtr GetMetricsDelegate(IntPtr ptr);
    private delegate IntPtr TakeStateDelegate(IntPtr ptr, byte[] json);
    private delegate IntPtr CheckEnabledDelegate(IntPtr ptr, byte[] toggle_name, byte[] context, byte[] customStrategyResults);
    private delegate IntPtr CheckVariantDelegate(IntPtr ptr, byte[] toggle_name, byte[] context, byte[] customStrategyResults);
    private delegate void FreeResponseDelegate(IntPtr ptr);
    private delegate IntPtr CountToggleDelegate(IntPtr ptr, byte[] toggle_name, bool enabled);
    private delegate IntPtr CountVariantDelegate(IntPtr ptr, byte[] toggle_name, byte[] variant_name);
    private delegate IntPtr ShouldEmitImpressionEventDelegate(IntPtr ptr, byte[] toggle_name);
    private delegate IntPtr BuiltInStrategiesDelegate(IntPtr ptr);
    private delegate IntPtr ListKnownTogglesDelegate(IntPtr ptr);
    private static readonly NewEngineDelegate new_engine;
    private static readonly FreeEngineDelegate free_engine;
    private static readonly GetMetricsDelegate get_metrics;
    private static readonly TakeStateDelegate take_state;
    private static readonly CheckEnabledDelegate check_enabled;
    private static readonly CheckVariantDelegate check_variant;
    private static readonly FreeResponseDelegate free_response;
    private static readonly CountToggleDelegate count_toggle;
    private static readonly CountVariantDelegate count_variant;
    private static readonly ShouldEmitImpressionEventDelegate should_emit_impression_event;
    private static readonly BuiltInStrategiesDelegate built_in_strategies;
    private static readonly ListKnownTogglesDelegate list_known_toggles;

    public static IntPtr NewEngine()
    {
        return new_engine();
    }

    public static void FreeEngine(IntPtr ptr)
    {
        free_engine(ptr);
    }

    public static IntPtr GetMetrics(IntPtr ptr)
    {
        return get_metrics(ptr);
    }

    public static IntPtr TakeState(IntPtr ptr, string json)
    {
        return take_state(ptr, ToUtf8Bytes(json));
    }

    public static IntPtr CheckEnabled(IntPtr ptr, string toggle_name, string context, string customStrategyResults)
    {
        return check_enabled(ptr, ToUtf8Bytes(toggle_name), ToUtf8Bytes(context), ToUtf8Bytes(customStrategyResults));
    }

    public static IntPtr CheckVariant(IntPtr ptr, string toggle_name, string context, string customStrategyResults)
    {
        return check_variant(ptr, ToUtf8Bytes(toggle_name), ToUtf8Bytes(context), ToUtf8Bytes(customStrategyResults));
    }

    public static void FreeResponse(IntPtr ptr)
    {
        free_response(ptr);
    }

    public static IntPtr CountToggle(IntPtr ptr, string toggle_name, bool enabled)
    {
        return count_toggle(ptr, ToUtf8Bytes(toggle_name), enabled);
    }

    public static IntPtr CountVariant(IntPtr ptr, string toggle_name, string variant_name)
    {
        return count_variant(ptr, ToUtf8Bytes(toggle_name), ToUtf8Bytes(variant_name));
    }

    public static IntPtr ShouldEmitImpressionEvent(IntPtr ptr, string toggle_name)
    {
        return should_emit_impression_event(ptr, ToUtf8Bytes(toggle_name));
    }

    public static IntPtr BuiltInStrategies(IntPtr ptr)
    {
        return built_in_strategies(ptr);
    }

    public static IntPtr ListKnownToggles(IntPtr ptr)
    {
        return list_known_toggles(ptr);
    }

    private static byte[] ToUtf8Bytes(string input)
    {
        byte[] utf8Bytes = System.Text.Encoding.UTF8.GetBytes(input);
        Array.Resize(ref utf8Bytes, utf8Bytes.Length + 1);
        return utf8Bytes;
    }
}
