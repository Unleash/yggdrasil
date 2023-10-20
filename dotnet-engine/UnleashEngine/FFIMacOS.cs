
using System.Runtime.InteropServices;


namespace Unleash;

internal class FFIMacOS : IFFIAccess {
    private const string DLL_PATH = "libyggdrasilffi.dylib";

    public IntPtr NewEngine() {
        return new_engine();
    }

    public void FreeEngine(IntPtr ptr)
    {
        free_engine(ptr);
    }

    public IntPtr GetMetrics(IntPtr ptr) {
        return get_metrics(ptr);
    }

    public IntPtr TakeState(IntPtr ptr, string json)
    {
        return take_state(ptr, json);
    }

    public IntPtr CheckEnabled(IntPtr ptr, string toggle_name, string context)
    {
        return check_enabled(ptr, toggle_name, context);
    }

    public IntPtr CheckVariant(IntPtr ptr, string toggle_name, string context) 
    {
        return check_variant(ptr, toggle_name, context);
    }

    public void FreeResponse(IntPtr ptr)
    {
        free_response(ptr);
    }

    public void CountToggle(IntPtr ptr, string toggle_name, bool enabled)
    {
        count_toggle(ptr, toggle_name, enabled);
    }

    public void CountVariant(IntPtr ptr, string toggle_name, string variant_name)
    {
        count_variant(ptr, toggle_name, variant_name);
    }

    [DllImport(DLL_PATH)]
    internal static extern IntPtr new_engine();

    [DllImport(DLL_PATH)]
    internal static extern void free_engine(IntPtr ptr);

    [DllImport(DLL_PATH)]
    internal static extern IntPtr take_state(IntPtr ptr, string json);

    [DllImport(DLL_PATH)]
    internal static extern IntPtr check_enabled(IntPtr ptr, string toggle_name, string context);

    [DllImport(DLL_PATH)]
    internal static extern IntPtr check_variant(IntPtr ptr, string toggle_name, string context);

    [DllImport(DLL_PATH)]
    internal static extern void free_response(IntPtr ptr);

    [DllImport(DLL_PATH)]
    internal static extern void count_toggle(IntPtr ptr, string toggle_name, bool enabled);

    [DllImport(DLL_PATH)]
    internal static extern void count_variant(IntPtr ptr, string toggle_name, string variant_name);

    [DllImport(DLL_PATH)]
    internal static extern IntPtr get_metrics(IntPtr ptr);
}
