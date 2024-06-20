using System.Runtime.InteropServices;

namespace Yggdrasil;

internal static class FFI
{

    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr new_engine();
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern void free_engine(IntPtr ptr);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr get_metrics(IntPtr ptr);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr take_state(IntPtr ptr, string json);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr check_enabled(IntPtr ptr, string toggle_name, string context, string customStrategyResults);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr check_variant(IntPtr ptr, string toggle_name, string context, string customStrategyResults);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern void free_response(IntPtr ptr);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern void count_toggle(IntPtr ptr, string toggle_name, bool enabled);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern void count_variant(IntPtr ptr, string toggle_name, string variant_name);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr should_emit_impression_event(IntPtr ptr, string toggle_name);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr built_in_strategies(IntPtr ptr);

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
        return take_state(ptr, json);
    }

    public static IntPtr CheckEnabled(
        IntPtr ptr,
        string toggle_name,
        string context,
        string customStrategyResults
    )
    {
        return check_enabled(ptr, toggle_name, context, customStrategyResults);
    }

    public static IntPtr CheckVariant(
        IntPtr ptr,
        string toggle_name,
        string context,
        string customStrategyResults
    )
    {
        return check_variant(ptr, toggle_name, context, customStrategyResults);
    }

    public static void FreeResponse(IntPtr ptr)
    {
        free_response(ptr);
    }

    public static void CountToggle(IntPtr ptr, string toggle_name, bool enabled)
    {
        count_toggle(ptr, toggle_name, enabled);
    }

    public static void CountVariant(IntPtr ptr, string toggle_name, string variant_name)
    {
        count_variant(ptr, toggle_name, variant_name);
    }

    public static IntPtr ShouldEmitImpressionEvent(IntPtr ptr, string toggle_name)
    {
        return should_emit_impression_event(ptr, toggle_name);
    }

    public static IntPtr BuiltInStrategies(IntPtr ptr)
    {
        return built_in_strategies(ptr);
    }
}
