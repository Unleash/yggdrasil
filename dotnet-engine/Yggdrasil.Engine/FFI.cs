using System.Reflection;
using System.Runtime.InteropServices;

namespace Yggdrasil;

internal static class FFI
{
  private delegate IntPtr NewEngineDelegate();
  private delegate void FreeEngineDelegate(IntPtr ptr);
  private delegate IntPtr GetMetricsDelegate(IntPtr ptr);
  private delegate IntPtr TakeStateDelegate(IntPtr ptr, string json);
  private delegate IntPtr CheckEnabledDelegate(
      IntPtr ptr,
      string toggle_name,
      string context,
      string customStrategyResults
  );
  private delegate IntPtr CheckVariantDelegate(
      IntPtr ptr,
      string toggle_name,
      string context,
      string customStrategyResults
  );
  private delegate void FreeResponseDelegate(IntPtr ptr);
  private delegate void CountToggleDelegate(IntPtr ptr, string toggle_name, bool enabled);
  private delegate void CountVariantDelegate(IntPtr ptr, string toggle_name, string variant_name);
  private delegate IntPtr ShouldEmitImpressionEventDelegate(IntPtr ptr, string toggle_name);

  private static readonly NewEngineDelegate _newEngine;
  private static readonly FreeEngineDelegate _freeEngine;
  private static readonly GetMetricsDelegate _getMetrics;
  private static readonly TakeStateDelegate _take_state;
  private static readonly CheckEnabledDelegate _check_enabled;
  private static readonly CheckVariantDelegate _check_variant;
  private static readonly FreeResponseDelegate _free_response;
  private static readonly CountToggleDelegate _count_toggle;
  private static readonly CountVariantDelegate _count_variant;
  private static readonly ShouldEmitImpressionEventDelegate _should_emit_impression_event;

  static FFI()
  {
    string dllPath = GetLibraryPath();
    IntPtr libHandle = NativeLibrary.Load(dllPath);

    _newEngine = Marshal.GetDelegateForFunctionPointer<NewEngineDelegate>(
        NativeLibrary.GetExport(libHandle, "new_engine")
    );

    _freeEngine = Marshal.GetDelegateForFunctionPointer<FreeEngineDelegate>(
        NativeLibrary.GetExport(libHandle, "free_engine")
    );

    _getMetrics = Marshal.GetDelegateForFunctionPointer<GetMetricsDelegate>(
        NativeLibrary.GetExport(libHandle, "get_metrics")
    );

    _take_state = Marshal.GetDelegateForFunctionPointer<TakeStateDelegate>(
        NativeLibrary.GetExport(libHandle, "take_state")
    );

    _check_enabled = Marshal.GetDelegateForFunctionPointer<CheckEnabledDelegate>(
        NativeLibrary.GetExport(libHandle, "check_enabled")
    );

    _check_variant = Marshal.GetDelegateForFunctionPointer<CheckVariantDelegate>(
        NativeLibrary.GetExport(libHandle, "check_variant")
    );

    _free_response = Marshal.GetDelegateForFunctionPointer<FreeResponseDelegate>(
        NativeLibrary.GetExport(libHandle, "free_response")
    );

    _count_toggle = Marshal.GetDelegateForFunctionPointer<CountToggleDelegate>(
        NativeLibrary.GetExport(libHandle, "count_toggle")
    );

    _count_variant = Marshal.GetDelegateForFunctionPointer<CountVariantDelegate>(
        NativeLibrary.GetExport(libHandle, "count_variant")
    );

    _should_emit_impression_event = Marshal.GetDelegateForFunctionPointer<ShouldEmitImpressionEventDelegate>(
        NativeLibrary.GetExport(libHandle, "should_emit_impression_event")
    );
  }

  private static string GetLibraryPath()
  {
    string libraryName;
    if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
      libraryName = "libyggdrasilffi.dll";
    else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
      libraryName = "libyggdrasilffi.so";
    else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
      libraryName = "libyggdrasilffi.dylib";
    else
      throw new PlatformNotSupportedException("Unsupported platform");

    var assemblyLocation = Assembly.GetExecutingAssembly().Location;
    if (assemblyLocation == null)
      throw new PlatformNotSupportedException("Unsupported platform");

    var assemblyDirectory = Path.GetDirectoryName(assemblyLocation);
    if (assemblyDirectory == null)
      throw new PlatformNotSupportedException("Unsupported platform");

    return Path.Combine(assemblyDirectory, libraryName);
  }

  public static IntPtr NewEngine()
  {
    return _newEngine();
  }

  public static void FreeEngine(IntPtr ptr)
  {
    _freeEngine(ptr);
  }

  public static IntPtr GetMetrics(IntPtr ptr)
  {
    return _getMetrics(ptr);
  }

  public static IntPtr TakeState(IntPtr ptr, string json)
  {
    return _take_state(ptr, json);
  }

  public static IntPtr CheckEnabled(
      IntPtr ptr,
      string toggle_name,
      string context,
      string customStrategyResults
  )
  {
    return _check_enabled(ptr, toggle_name, context, customStrategyResults);
  }

  public static IntPtr CheckVariant(
      IntPtr ptr,
      string toggle_name,
      string context,
      string customStrategyResults
  )
  {
    return _check_variant(ptr, toggle_name, context, customStrategyResults);
  }

  public static void FreeResponse(IntPtr ptr)
  {
    _free_response(ptr);
  }

  public static void CountToggle(IntPtr ptr, string toggle_name, bool enabled)
  {
    _count_toggle(ptr, toggle_name, enabled);
  }

  public static void CountVariant(IntPtr ptr, string toggle_name, string variant_name)
  {
    _count_variant(ptr, toggle_name, variant_name);
  }

  public static IntPtr ShouldEmitImpressionEvent(IntPtr ptr, string toggle_name)
  {
    return _should_emit_impression_event(ptr, toggle_name);
  }
}
