namespace Yggdrasil;

internal interface IFFIAccess {
    IntPtr NewEngine();

    IntPtr GetMetrics(IntPtr ptr);

    IntPtr TakeState(IntPtr ptr, string json);

    IntPtr CheckEnabled(IntPtr ptr, string toggle_name, string context, string customStrategyResults);

    IntPtr CheckVariant(IntPtr ptr, string toggle_name, string context, string customStrategyResults);

    void FreeEngine(IntPtr ptr);

    void FreeResponse(IntPtr ptr);

    void CountToggle(IntPtr ptr, string toggle_name, bool enabled);

    void CountVariant(IntPtr ptr, string toggle_name, string variant_name);
}
