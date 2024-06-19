using System.Runtime.InteropServices;

static class NativeLibraryWindowsHelper
{
    [DllImport("kernel32", SetLastError = true)]
    private static extern IntPtr LoadLibraryWindows(string dllToLoad);

    [DllImport("kernel32", SetLastError = true)]
    private static extern bool FreeLibraryWindows(IntPtr handle);

    [DllImport("kernel32", SetLastError = true)]
    private static extern IntPtr GetProcAddressWindows(IntPtr hModule, string procedureName);

    public static IntPtr Load(string libraryPath)
    {
        return LoadLibraryWindows(libraryPath);
    }

    public static void Free(IntPtr handle)
    {
        FreeLibraryWindows(handle);
    }

    public static IntPtr GetExport(IntPtr handle, string name)
    {
        return GetProcAddressWindows(handle, name);
    }
}

static class NativeLibraryLinuxHelper
{
    [DllImport("libdl.so.2", SetLastError = true)]
    private static extern IntPtr dlopen(string fileName, int flags);

    [DllImport("libdl.so.2", SetLastError = true)]
    private static extern int dlclose(IntPtr handle);

    [DllImport("libdl.so.2", SetLastError = true)]
    private static extern IntPtr dlsym(IntPtr handle, string name);

    [DllImport("libdl.so.2", SetLastError = true)]
    private static extern IntPtr dlerror();

    public static IntPtr Load(string libraryPath)
    {
        const int RTLD_NOW = 2;
        IntPtr handle = dlopen(libraryPath, RTLD_NOW);

        if (handle == IntPtr.Zero)
        {
            IntPtr errorPtr = dlerror();
            string errorMessage = Marshal.PtrToStringAuto(errorPtr);
            throw new InvalidOperationException($"Failed to load library {libraryPath}: {errorMessage}");
        }

        return handle;
    }

    public static void Free(IntPtr handle)
    {
        dlclose(handle);
    }

    public static IntPtr GetExport(IntPtr handle, string name)
    {
        dlerror();
        IntPtr res = dlsym(handle, name);
        IntPtr errorPtr = dlerror();
        if (errorPtr != IntPtr.Zero)
        {
            string errorMessage = Marshal.PtrToStringAuto(errorPtr);
            throw new InvalidOperationException($"Failed to get function pointer for {name}: {errorMessage}");
        }
        return res;
    }
}

static class NativeLibraryOSXHelper
{
    [DllImport("libc.dylib", SetLastError = true)]
    private static extern IntPtr dlopen(string fileName, int flags);

    [DllImport("libc.dylib", SetLastError = true)]
    private static extern int dlclose(IntPtr handle);

    [DllImport("libc.dylib", SetLastError = true)]
    private static extern IntPtr dlsym(IntPtr handle, string name);

    [DllImport("libc.dylib", SetLastError = true)]
    private static extern IntPtr dlerror();

    public static IntPtr Load(string libraryPath)
    {
        const int RTLD_NOW = 2;
        IntPtr handle = dlopen(libraryPath, RTLD_NOW);

        if (handle == IntPtr.Zero)
        {
            IntPtr errorPtr = dlerror();
            string errorMessage = Marshal.PtrToStringAuto(errorPtr);
            throw new InvalidOperationException($"Failed to load library {libraryPath}: {errorMessage}");
        }

        return handle;
    }

    public static void Free(IntPtr handle)
    {
        dlclose(handle);
    }

    public static IntPtr GetExport(IntPtr handle, string name)
    {
        dlerror();
        IntPtr res = dlsym(handle, name);
        IntPtr errorPtr = dlerror();
        if (errorPtr != IntPtr.Zero)
        {
            string errorMessage = Marshal.PtrToStringAuto(errorPtr);
            throw new InvalidOperationException($"Failed to get function pointer for {name}: {errorMessage}");
        }
        return res;
    }
}

public static class NativeLibraryHelper
{
    public static IntPtr Load(string libraryPath)
    {
        IntPtr handle = IntPtr.Zero;

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            handle = NativeLibraryWindowsHelper.Load(libraryPath);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            handle = NativeLibraryLinuxHelper.Load(libraryPath);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            handle = NativeLibraryOSXHelper.Load(libraryPath);
        }
        else
        {
            throw new PlatformNotSupportedException();
        }

        if (handle == IntPtr.Zero)
        {
            throw new InvalidOperationException($"Failed to load library {libraryPath}");
        }

        return handle;
    }

    public static void Free(IntPtr handle)
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            NativeLibraryWindowsHelper.Free(handle);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            NativeLibraryLinuxHelper.Free(handle);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            NativeLibraryOSXHelper.Free(handle);
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
            functionPointer = NativeLibraryWindowsHelper.GetExport(handle, name);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            functionPointer = NativeLibraryLinuxHelper.GetExport(handle, name);
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            functionPointer = NativeLibraryOSXHelper.GetExport(handle, name);
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

}
