namespace Yggdrasil;

internal interface IFFIAccess {
    IntPtr NewEngine();

    IntPtr GetMetrics(IntPtr ptr);

    IntPtr TakeState(IntPtr ptr, string json);

    IntPtr CheckEnabled(IntPtr ptr, string toggle_name, string context);

    IntPtr CheckVariant(IntPtr ptr, string toggle_name, string context);

    void FreeEngine(IntPtr ptr);

    void FreeResponse(IntPtr ptr);

    void CountToggle(IntPtr ptr, string toggle_name, bool enabled);

    void CountVariant(IntPtr ptr, string toggle_name, string variant_name);
}
