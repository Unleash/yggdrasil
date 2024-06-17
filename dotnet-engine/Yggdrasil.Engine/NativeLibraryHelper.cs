using System.Runtime.InteropServices;

public static class NativeLibraryHelper
{
    public static IntPtr Load(string libraryPath)
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            return LoadLibraryWindows(libraryPath);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            return LoadLibraryLinux(libraryPath);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            return LoadLibraryMac(libraryPath);
        }
        else
        {
            throw new PlatformNotSupportedException();
        }
    }

    public static void Free(IntPtr handle)
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            FreeLibraryWindows(handle);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux) ||
                 RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            FreeLibraryUnix(handle);
        }
        else
        {
            throw new PlatformNotSupportedException();
        }
    }

    public static IntPtr GetExport(IntPtr handle, string name)
    {
        IntPtr functionPointer;
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            functionPointer = GetProcAddressWindows(handle, name);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux) ||
                 RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            functionPointer = GetProcAddressUnix(handle, name);
        }
        else
        {
            throw new PlatformNotSupportedException();
        }

        if (functionPointer == IntPtr.Zero)
        {
            throw new InvalidOperationException($"Failed to get function pointer for {name}");
        }

        return functionPointer;
    }

    [DllImport("kernel32", SetLastError = true)]
    private static extern IntPtr LoadLibraryWindows(string dllToLoad);

    [DllImport("kernel32", SetLastError = true)]
    private static extern bool FreeLibraryWindows(IntPtr handle);

    [DllImport("kernel32", SetLastError = true)]
    private static extern IntPtr GetProcAddressWindows(IntPtr hModule, string procedureName);

    [DllImport("libdl", SetLastError = true)]
    private static extern IntPtr dlopen(string fileName, int flags);

    [DllImport("libdl", SetLastError = true)]
    private static extern int dlclose(IntPtr handle);

    [DllImport("libdl", SetLastError = true)]
    private static extern IntPtr dlsym(IntPtr handle, string name);

    [DllImport("libdl", SetLastError = true)]
    private static extern IntPtr dlerror();

    private static IntPtr LoadLibraryLinux(string dllToLoad)
    {
        const int RTLD_NOW = 2;
        IntPtr handle = dlopen(dllToLoad, RTLD_NOW);
        if (handle == IntPtr.Zero)
        {
            IntPtr errorPtr = dlerror();
            string errorMessage = Marshal.PtrToStringAnsi(errorPtr);
            throw new InvalidOperationException($"Failed to load library {dllToLoad}: {errorMessage}");
        }
        return handle;
    }

    private static IntPtr LoadLibraryMac(string dllToLoad)
    {
        return LoadLibraryLinux(dllToLoad);
    }

    private static int FreeLibraryUnix(IntPtr handle)
    {
        return dlclose(handle);
    }

    private static IntPtr GetProcAddressUnix(IntPtr handle, string name)
    {
        dlerror();
        IntPtr res = dlsym(handle, name);
        IntPtr errorPtr = dlerror();
        if (errorPtr != IntPtr.Zero)
        {
            string errorMessage = Marshal.PtrToStringAnsi(errorPtr);
            throw new InvalidOperationException($"Failed to get function pointer for {name}: {errorMessage}");
        }
        return res;
    }
}
