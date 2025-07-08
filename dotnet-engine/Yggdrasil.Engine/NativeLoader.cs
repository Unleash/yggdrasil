using System.Reflection;
using System.Runtime.InteropServices;

internal static class NativeLibLoader
{
    internal static IntPtr LoadNativeLibrary()
    {
        var libName = GetBinaryName();
        var tempPath = Path.Combine(Path.GetTempPath(), libName);
        var assembly = Assembly.GetExecutingAssembly();
        var assemblyName = assembly.GetName().Name;

        if (!File.Exists(tempPath))
        {
            using (var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream($"{assemblyName}.{libName}"))
            {
                if (stream == null)
                    throw new FileNotFoundException($"Embedded resource {libName} not found.");

                using (var fileStream = new FileStream(tempPath, FileMode.Create, FileAccess.Write))
                {
                    stream.CopyTo(fileStream);
                }
            }
        }

        return LoadBinary(tempPath);
    }

    private static string GetBinaryName()
    {
        string os, arch, libc = "";

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            os = "win";
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            os = "linux";
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            os = "osx";
        else
            throw new PlatformNotSupportedException("Unsupported OS");

        if (RuntimeInformation.ProcessArchitecture == Architecture.X64)
            arch = "x86_64";
        else if (RuntimeInformation.ProcessArchitecture == Architecture.Arm64)
            arch = "arm64";
        else if (RuntimeInformation.ProcessArchitecture == Architecture.X86)
            arch = IntPtr.Size == 4 ? "i686" : "x86_64";
        else
            throw new PlatformNotSupportedException("Unsupported CPU architecture");

        if (os == "linux" && IsMusl())
            libc = "-musl";

        var versionAttribute = Assembly
            .GetExecutingAssembly()
            .GetCustomAttribute<YggdrasilCoreVersionAttribute>();
        if (versionAttribute == null)
            throw new InvalidOperationException("YggdrasilCoreVersionAttribute was not defined on the assembly.");
        var versionString = versionAttribute.Version;

        string filename = os == "win"
            ? $"yggdrasilffi_{arch}_{versionString}.dll"
            : $"libyggdrasilffi_{arch}{libc}_{versionString}.{(os == "osx" ? "dylib" : "so")}";

        return filename;
    }

    private static bool IsMusl()
    {
        if (!RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            return false;

        try
        {
            string output = File.ReadAllText("/proc/self/maps");
            return output.Contains("musl");
        }
        catch
        {
            try
            {
                using (var process = new System.Diagnostics.Process())
                {
                    process.StartInfo = new System.Diagnostics.ProcessStartInfo
                    {
                        FileName = "ldd",
                        Arguments = "--version",
                        RedirectStandardOutput = true,
                        UseShellExecute = false
                    };
                    process.Start();
                    string lddOutput = process.StandardOutput.ReadToEnd();
                    process.WaitForExit();

                    return lddOutput.Contains("musl");
                }
            }
            catch
            {
                return false;
            }
        }
    }

    internal static IntPtr LoadFunctionPointer(IntPtr libHandle, string functionName)
    {
        var nativeLibraryType = Type.GetType("System.Runtime.InteropServices.NativeLibrary, System.Runtime.InteropServices");
        if (nativeLibraryType != null)
        {
            var getExportMethod = nativeLibraryType.GetMethod("GetExport", new[] { typeof(IntPtr), typeof(string) });
            if (getExportMethod != null)
            {
                var invocationPtr = (IntPtr?)getExportMethod.Invoke(null, new object[] { libHandle, functionName });
                if (invocationPtr.HasValue && invocationPtr.Value != IntPtr.Zero)
                    return invocationPtr.Value;
            }
        }

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            return GetProcAddress(libHandle, functionName);
        else
            return dlsym(libHandle, functionName);
    }

    private static IntPtr LoadBinary(string libPath)
    {
        IntPtr? handle;
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            handle = LoadWindowsLibrary(libPath);
        else
            handle = LoadUnixLibrary(libPath);

        if (handle is null || handle == IntPtr.Zero)
            throw new DllNotFoundException($"Failed to load library from {libPath}");

        return (IntPtr)handle;
    }

    private static IntPtr LoadUnixLibrary(string libPath)
    {
        // Try NativeLibrary.Load (works on .NET Core 3+)
        var nativeLibraryType = Type.GetType("System.Runtime.InteropServices.NativeLibrary, System.Runtime.InteropServices");
        if (nativeLibraryType != null)
        {
            var loadMethod = nativeLibraryType.GetMethod("Load", new[] { typeof(string) });
            if (loadMethod != null)
            {
                var invocationPtr = (IntPtr?)loadMethod.Invoke(null, new object[] { libPath });
                if (invocationPtr.HasValue && invocationPtr.Value != IntPtr.Zero)
                    return invocationPtr.Value;
            }
        }
        return dlopen(libPath, RTLD_NOW);
    }

    private static IntPtr LoadWindowsLibrary(string libPath)
    {
        // Try NativeLibrary.Load (works on .NET Core 3+)
        var nativeLibraryType = Type.GetType("System.Runtime.InteropServices.NativeLibrary, System.Runtime.InteropServices");
        if (nativeLibraryType != null)
        {
            var loadMethod = nativeLibraryType.GetMethod("Load", new[] { typeof(string) });
            if (loadMethod != null)
            {
                var invocationPtr = (IntPtr?)loadMethod.Invoke(null, new object[] { libPath });
                if (invocationPtr.HasValue && invocationPtr.Value != IntPtr.Zero)
                    return invocationPtr.Value;
            }
        }

        return LoadLibrary(libPath);
    }

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern IntPtr LoadLibrary(string dllToLoad);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool FreeLibrary(IntPtr hModule);

    [DllImport("libdl.so.2", EntryPoint = "dlopen")]
    private static extern IntPtr dlopen(string filename, int flags);

    [DllImport("libdl.so.2", EntryPoint = "dlclose")]
    private static extern int dlclose(IntPtr handle);

    private const int RTLD_NOW = 2;

    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Ansi)]
    private static extern IntPtr GetProcAddress(IntPtr hModule, string procName);

    [DllImport("libdl.so.2", SetLastError = true, CharSet = CharSet.Ansi)]
    private static extern IntPtr dlsym(IntPtr handle, string symbol);
}
