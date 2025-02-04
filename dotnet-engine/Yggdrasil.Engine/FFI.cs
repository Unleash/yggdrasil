using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;

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
    private static extern IntPtr take_state(IntPtr ptr, byte[] json);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr check_enabled(IntPtr ptr, byte[] toggle_name, byte[] context, byte[] customStrategyResults);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr check_variant(IntPtr ptr, byte[] toggle_name, byte[] context, byte[] customStrategyResults);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern void free_response(IntPtr ptr);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr count_toggle(IntPtr ptr, byte[] toggle_name, bool enabled);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr count_variant(IntPtr ptr, byte[] toggle_name, byte[] variant_name);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr should_emit_impression_event(IntPtr ptr, byte[] toggle_name);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr built_in_strategies(IntPtr ptr);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr list_known_toggles(IntPtr ptr);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern bool quick_check(IntPtr ptr, byte[] message, int messageLength);


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

    public static bool QuickCheck(IntPtr ptr,
        string toggleName,
        Context context,
        Dictionary<string, bool>? customStrategyResults)
    {
        byte[] message = PackMessage(context, customStrategyResults);

        return quick_check(ptr, message, message.Length);
    }

    public static IntPtr CheckEnabled(
        IntPtr ptr,
        string toggle_name,
        string context,
        string customStrategyResults
    )
    {
        return check_enabled(ptr, ToUtf8Bytes(toggle_name), ToUtf8Bytes(context), ToUtf8Bytes(customStrategyResults));
    }

    public static IntPtr CheckVariant(
        IntPtr ptr,
        string toggle_name,
        string context,
        string customStrategyResults
    )
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

    /// <summary>
    /// Converts a string to a UTF-8 encoded null-terminated byte array.
    /// </summary>
    /// <param name="input">The string to convert.</param>
    /// <returns>A UTF-8 encoded null-terminated byte array.</returns>
    private static byte[] ToUtf8Bytes(string input)
    {
        byte[] utf8Bytes = System.Text.Encoding.UTF8.GetBytes(input);
        Array.Resize(ref utf8Bytes, utf8Bytes.Length + 1);
        return utf8Bytes;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct ContextHeader
    {
        public uint user_id_offset;
        public uint session_id_offset;
        public uint remote_address_offset;
        public uint environment_offset;
        public uint app_name_offset;
    }

    public static byte[] PackMessage(Context ctx, Dictionary<string, bool>? customStrategies)
    {
        // Precompute the required buffer size
        int headerSize = Marshal.SizeOf<ContextHeader>();
        int stringDataSize = (
            (ctx.UserId?.Length ?? 0) +
            (ctx.SessionId?.Length ?? 0) +
            (ctx.RemoteAddress?.Length ?? 0) +
            (ctx.Environment?.Length ?? 0) +
            (ctx.AppName?.Length ?? 0)
        );

        // Allocate buffer **once**
        byte[] buffer = new byte[headerSize + stringDataSize];

        // Unsafe write of the header directly into the buffer
        ref ContextHeader header = ref Unsafe.As<byte, ContextHeader>(ref buffer[0]);

        // Track string offsets
        int currentOffset = headerSize;

        int WriteString(string? s)
        {
            if (string.IsNullOrEmpty(s)) return 0;
            int offset = currentOffset;
            Encoding.UTF8.GetBytes(s, 0, s!.Length, buffer, offset);
            currentOffset += s.Length;
            return offset;
        }

        // Write offsets into the header
        header.user_id_offset = (uint)WriteString(ctx.UserId);
        header.session_id_offset = (uint)WriteString(ctx.SessionId);
        header.remote_address_offset = (uint)WriteString(ctx.RemoteAddress);
        header.environment_offset = (uint)WriteString(ctx.Environment);
        header.app_name_offset = (uint)WriteString(ctx.AppName);

        return buffer;
    }
}
