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
    public static TRead? ReadPrimitive<TRead>(IntPtr ptr) where TRead : struct
    {
        var engineResponse = ReadEngineResponse<EngineResponse<TRead>>(ptr);
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
    public static TRead? ReadComplex<TRead>(IntPtr ptr) where TRead : class
    {
        var engineResponse = ReadEngineResponse<EngineResponse<TRead>>(ptr);
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
        var engineResponse = ReadEngineResponse<EngineResponse>(ptr);
        if (engineResponse?.StatusCode == "Error")
        {
            throw new YggdrasilEngineException($"Error: {engineResponse?.ErrorMessage}");
        }
    }

    private static TEngineResponse? ReadEngineResponse<TEngineResponse>(IntPtr ptr) where TEngineResponse : EngineResponse
    {
        var json = Marshal.PtrToStringUTF8(ptr);

        FFI.FreeResponse(ptr);

        var result = json != null
            ? JsonSerializer.Deserialize<TEngineResponse>(json, options)
            : null;

        return result;
    }
}