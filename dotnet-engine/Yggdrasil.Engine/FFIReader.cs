namespace Yggdrasil;
using System.Runtime.InteropServices;
using System.Text.Json;

public static class FFIReader
{
    private static JsonSerializerOptions options = new JsonSerializerOptions
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    /// <summary>
    /// Handles response from engine and deserializes value to a primitive type (bool, int, string, etc).
    /// Throws <see cref="YggdrasilEngineException"/> if engine response is an error.
    /// Returns null if engine response is null.
    /// Free's the response pointer.
    /// </summary>
    /// <typeparam name="TRead">The type of object to deserialize to and return</typeparam>
    /// <param name="ptr">Pointer to a string containing the response from FFI. This pointer will be freed</param>
    /// <returns>The result from deserializing the engine response</returns>
    /// <exception cref="YggdrasilEngineException"></exception>
    public static TRead? ReadPrimitive<TRead>(IntPtr ptr)
        where TRead : struct
    {
        if (ptr == IntPtr.Zero)
        {
            return null;
        }

        var engineResponse = ReadResponse<EngineResponse<TRead?>>(ptr);
        if (engineResponse?.StatusCode == "Error")
        {
            throw new YggdrasilEngineException($"Error: {engineResponse?.ErrorMessage}");
        }

        return engineResponse?.Value;
    }

    /// <summary>
    /// Handles response from engine and deserializes value to a complex type (class).
    /// Throws <see cref="YggdrasilEngineException"/> if engine response is an error.
    /// Returns null if engine response is null.
    /// Free's the response pointer.
    /// </summary>
    /// <typeparam name="TRead">The type of object to deserialize to and return</typeparam>
    /// <param name="ptr">Pointer to a string containing the response from FFI. This pointer will be freed</param>
    /// <returns>The result from deserializing the engine response</returns>
    /// <exception cref="YggdrasilEngineException"></exception>
    public static TRead? ReadComplex<TRead>(IntPtr ptr)
        where TRead : class
    {
        if (ptr == IntPtr.Zero)
        {
            return null;
        }

        var engineResponse = ReadResponse<EngineResponse<TRead>>(ptr);
        if (engineResponse?.StatusCode == "Error")
        {
            throw new YggdrasilEngineException($"Error: {engineResponse?.ErrorMessage}");
        }

        return engineResponse?.Value;
    }

    /// <summary>
    /// Handles the result of an engine operation that does not return a value.
    /// Throws <see cref="YggdrasilEngineException"/> if engine response is an error.
    /// Free's the response pointer.
    /// </summary>
    /// <param name="ptr">Pointer to a string containing the response from FFI. This pointer will be freed</param>
    /// <exception cref="YggdrasilEngineException"></exception>
    public static void CheckResponse(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero)
        {
            throw new YggdrasilEngineException($"Error: unexpected null pointer");
        }

        var engineResponse = ReadResponse<EngineResponse>(ptr);
        if (engineResponse?.StatusCode == "Error")
        {
            throw new YggdrasilEngineException($"Error: {engineResponse?.ErrorMessage}");
        }
    }

    internal static T? ReadResponse<T>(IntPtr ptr)
        where T : class
    {
        if (ptr == IntPtr.Zero)
        {
            throw new YggdrasilEngineException($"Error: unexpected null pointer");
        }

        try
        {
            string? json;

            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
                json = Marshal.PtrToStringAnsi(ptr);
            else
                json = Marshal.PtrToStringAuto(ptr);

            var result = json != null ? JsonSerializer.Deserialize<T>(json, options) : null;

            return result;
        }
        finally
        {
            FFI.FreeResponse(ptr);
        }
    }
}
