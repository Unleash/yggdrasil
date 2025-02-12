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
    private static extern EnabledMessage quick_check(IntPtr ptr, byte[] message, int messageLength);
    [DllImport("yggdrasilffi", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern void free_enabled_response(ref EnabledMessage message);


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

    internal unsafe static bool? QuickCheck(IntPtr ptr,
        string toggleName,
        Context context,
        Dictionary<string, bool>? customStrategyResults)
    {
        byte[] requestBuffer = PackMessage(toggleName, context, customStrategyResults);

        fixed (byte* requestPtr = requestBuffer)
        {
            EnabledMessage response = quick_check(ptr, requestBuffer, requestBuffer.Length);

            try
            {
                if (response.error != IntPtr.Zero)
                {
                    string errorMsg = Marshal.PtrToStringAnsi(response.error);
                    throw new Exception($"Rust error: {errorMsg}");
                }

                return response.value switch
                {
                    0 => false,
                    1 => true,
                    2 => (bool?)null,
                    _ => throw new Exception("Invalid Rust response")
                };
            }
            finally
            {
                free_enabled_response(ref response);
            }
        }
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
        public uint current_time_offset;
        public uint app_name_offset;
        public uint properties_offset;
        public uint properties_count;
        public uint custom_strategies_offset;
        public uint custom_strategies_count;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct EnabledMessage
    {
        public byte value;
        public IntPtr error;
    }

    const int CUSTOM_STRATEGY_ENTRY_SIZE = sizeof(uint) + sizeof(byte);
    const int PROPERTY_ENTRY_SIZE = sizeof(uint) * 2;

    private static byte[] PackMessage(string toggleName, Context ctx, Dictionary<string, bool>? customStrategies)
    {
        int GetUtf8ByteCount(string? s) => string.IsNullOrEmpty(s) ? 0 : Encoding.UTF8.GetByteCount(s) + 1; // +1 for null terminators

        // We can calculate the byte count of the buffer we need ahead of time
        int headerSize = Marshal.SizeOf<MessageHeader>();

        int propertiesCount = ctx.Properties?.Count ?? 0;
        int propertiesTableSize = propertiesCount * sizeof(uint) * 2;
        int propertiesStringSize = ctx.Properties?.Sum(kvp => GetUtf8ByteCount(kvp.Key) + GetUtf8ByteCount(kvp.Value)) ?? 0;

        int customStrategiesCount = customStrategies?.Count ?? 0;
        int customStrategiesTableSize = customStrategiesCount * (sizeof(uint) + sizeof(byte));
        int customStrategiesStringSize = customStrategies?.Sum(kvp => GetUtf8ByteCount(kvp.Key)) ?? 0;

        string? dateTime = ctx.CurrentTime?.ToString("O");

        int fixedStringDataSize = (
            GetUtf8ByteCount(toggleName) +
            GetUtf8ByteCount(ctx.UserId) +
            GetUtf8ByteCount(ctx.SessionId) +
            GetUtf8ByteCount(ctx.RemoteAddress) +
            GetUtf8ByteCount(ctx.Environment) +
            GetUtf8ByteCount(dateTime) +
            GetUtf8ByteCount(ctx.AppName)
        );

        // Now we allocate that buffer **once**, this is quite expensive so we want to avoid doing it multiple times
        byte[] buffer = new byte[headerSize
            + fixedStringDataSize
            + propertiesTableSize
            + propertiesStringSize
            + customStrategiesTableSize
            + customStrategiesStringSize];

        // Unsafe write of the header directly into the buffer, this means changing properties on the header object will change the buffer
        ref MessageHeader header = ref Unsafe.As<byte, MessageHeader>(ref buffer[0]);

        int currentOffset = headerSize;

        uint WriteString(string? s)
        {
            if (string.IsNullOrEmpty(s)) return 0;
            int offset = currentOffset;
            int strLength = Encoding.UTF8.GetByteCount(s);

            Encoding.UTF8.GetBytes(s, 0, s!.Length, buffer, offset);
            buffer[currentOffset + strLength] = 0;  // Add null terminator
            currentOffset += strLength + 1;  // Move past the null terminator
            return (uint)offset;
        }

        header.toggle_name_offset = WriteString(toggleName);
        header.user_id_offset = WriteString(ctx.UserId);
        header.session_id_offset = WriteString(ctx.SessionId);
        header.remote_address_offset = WriteString(ctx.RemoteAddress);
        header.environment_offset = WriteString(ctx.Environment);
        header.current_time_offset = WriteString(dateTime);
        header.app_name_offset = WriteString(ctx.AppName);
        header.properties_count = (uint)propertiesCount;
        header.custom_strategies_count = (uint)customStrategiesCount;

        int propertiesOffset = currentOffset;
        header.properties_offset = (uint)propertiesOffset;
        // skip ahead by the size of the table, we'll write
        // the bytes in later as we get the offsets from WriteString
        currentOffset += propertiesTableSize;

        if (ctx.Properties != null && ctx.Properties.Count > 0)
        {
            // now we can write the properties table pair by pair
            // as we write the bytes with WriteString we can back fill
            // the table offsets
            for (var i = 0; i < propertiesCount; i++)
            {
                var kvp = ctx.Properties.ElementAt(i);

                uint keyPos = WriteString(kvp.Key);
                uint valuePos = WriteString(kvp.Value);
                BitConverter.GetBytes(keyPos).CopyTo(buffer, propertiesOffset + i * PROPERTY_ENTRY_SIZE);
                BitConverter.GetBytes(valuePos).CopyTo(buffer, propertiesOffset + i * PROPERTY_ENTRY_SIZE + sizeof(uint));
            }
        }

        int customStrategiesOffset = currentOffset;
        header.custom_strategies_offset = (uint)customStrategiesOffset;
        // skip ahead by the size of the table, we'll write
        // the bytes in later as we get the offsets from WriteString
        currentOffset += customStrategiesTableSize;
        if (customStrategies != null && customStrategies.Count > 0)
        {
            for (var i = 0; i < customStrategies.Count; i++)
            {
                var kvp = customStrategies.ElementAt(i);
                var keyOffset = WriteString(kvp.Key);

                BitConverter.GetBytes(keyOffset).CopyTo(buffer, customStrategiesOffset + (i * CUSTOM_STRATEGY_ENTRY_SIZE));
                buffer[customStrategiesOffset + (i * CUSTOM_STRATEGY_ENTRY_SIZE) + sizeof(uint)] = kvp.Value ? (byte)1 : (byte)0;
            }
        }

        return buffer;
    }
}
