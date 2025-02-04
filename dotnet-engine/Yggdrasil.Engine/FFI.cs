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
        byte[] message = PackMessage(toggleName, context, customStrategyResults);

        // return false;
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
    public struct MessageHeader
    {
        public uint toggle_name_offset;
        public uint user_id_offset;
        public uint session_id_offset;
        public uint remote_address_offset;
        public uint environment_offset;
        public uint app_name_offset;
    }

    private static byte[] PackMessage(string toggleName, Context ctx, Dictionary<string, bool>? customStrategies)
    {
        // Precompute the required buffer size, the +1 is space for the null terminators
        int headerSize = Marshal.SizeOf<MessageHeader>();
        int stringDataSize = (
            toggleName.Length + 1 +
            (ctx.UserId?.Length ?? 0) + 1 +
            (ctx.SessionId?.Length ?? 0) + 1 +
            (ctx.RemoteAddress?.Length ?? 0) + 1 +
            (ctx.Environment?.Length ?? 0) + 1 +
            (ctx.AppName?.Length ?? 0) + 1
        );

        // Now we allocate that buffer **once**, this is surprisingly expensive because everything else here is so cheap
        byte[] buffer = new byte[headerSize + stringDataSize];
        Console.WriteLine("AYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY" + buffer.Length);
        Console.WriteLine("Name is" + toggleName.Length);

        // Unsafe write of the header directly into the buffer
        ref MessageHeader header = ref Unsafe.As<byte, MessageHeader>(ref buffer[0]);

        int currentOffset = headerSize;

        int WriteString(string? s)
        {
            if (string.IsNullOrEmpty(s)) return 0;
            int offset = currentOffset;
            Encoding.UTF8.GetBytes(s, 0, s!.Length, buffer, offset);
            buffer[currentOffset + s.Length] = 0;  // Add null terminator
            currentOffset += s.Length + 1;  // Move past the null terminator
            Console.WriteLine("Writing string '" + s + "' with offset" + offset + " and length " + s.Length);
            return offset;
        }

        // Write offsets into the header
        header.toggle_name_offset = (uint)WriteString(toggleName);
        header.user_id_offset = (uint)WriteString(ctx.UserId);
        header.session_id_offset = (uint)WriteString(ctx.SessionId);
        header.remote_address_offset = (uint)WriteString(ctx.RemoteAddress);
        header.environment_offset = (uint)WriteString(ctx.Environment);
        header.app_name_offset = (uint)WriteString(ctx.AppName);

        return buffer;
    }
}
