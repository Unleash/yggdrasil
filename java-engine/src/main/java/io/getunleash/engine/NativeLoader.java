package io.getunleash.engine;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

import java.io.BufferedReader;
import java.io.FileNotFoundException;
import java.io.FileReader;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;
import java.lang.System;

interface UnleashFFI extends Library {

    Pointer newEngine();

    void freeEngine(Pointer ptr);

    Pointer takeState(Pointer ptr, Pointer toggles);

    Pointer checkEnabled(Pointer ptr, Pointer name, Pointer context, Pointer customStrategyResults);

    Pointer checkVariant(Pointer ptr, Pointer name, Pointer context, Pointer customStrategyResults);

    void countToggle(Pointer ptr, Pointer name, boolean enabled);

    void countVariant(Pointer ptr, Pointer name, Pointer variantName);

    Pointer getMetrics(Pointer ptr);

    Pointer shouldEmitImpressionEvent(Pointer ptr, Pointer name);

    Pointer builtInStrategies();

    void freeResponse(Pointer pointer);

    Pointer listKnownToggles(Pointer ptr);

    Pointer getCoreVersion();

    static UnleashFFI getInstance() {
        return NativeLoader.NATIVE_INTERFACE;
    }

    static Pointer getYggdrasilCoreVersion() {
        return NativeLoader.NATIVE_INTERFACE.getCoreVersion();
    }
}

class NativeLoader {
    static final UnleashFFI NATIVE_INTERFACE;
    static {
        NATIVE_INTERFACE = loadLibrary();
    }

    static UnleashFFI loadLibrary() {
        String os = System.getProperty("os.name").toLowerCase();
        String arch = System.getProperty("os.arch").toLowerCase();
        String libName;

        if (os.contains("mac")) {
            // Catches a case where some legacy mac machines report arm64 over aarch64
            if (arch.contains("aarch64") || arch.contains("arm64")) {
                libName = "libyggdrasilffi_arm64.dylib";
            } else {
                libName = "libyggdrasilffi_x86_64.dylib";
            }
        } else if (os.contains("win")) {
            if (arch.equals("x86_64") || arch.contains("amd64")) {
                libName = "yggdrasilffi_x86_64.dll";
            } else if (arch.equals("x86") || arch.equals("i386") || arch.equals("i686")) {
                libName = "yggdrasilffi_i686.dll";
            } else if (arch.contains("arm64")) {
                libName = "yggdrasilffi_arm64.dll";
            } else {
                throw new UnsupportedOperationException("Unsupported architecture on Windows: " + arch);
            }
        } else if (os.contains("linux")) {
            if (isMusl()) {
                if (arch.contains("aarch64")) {
                    libName = "libyggdrasilffi_arm64-musl.so ";
                } else {
                    libName = "libyggdrasilffi_x86_64-musl.so";
                }
            } else if (arch.contains("aarch64") || arch.contains("arm64")) {
                libName = "libyggdrasilffi_arm64.so";
            } else {
                libName = "libyggdrasilffi_x86_64.so";
            }
        } else {
            throw new UnsupportedOperationException("Unsupported operating system: " + os + ", architecture: " + arch);
        }

        Map<String, Object> options = new HashMap<>();
        options.put(Library.OPTION_FUNCTION_MAPPER, new CamelToSnakeMapper());
        options.put(Library.OPTION_STRING_ENCODING, "UTF-8");

        try {
            // Extract and load the native library from the JAR
            Path tempLib = extractLibraryFromJar(libName);
            System.load(tempLib.toAbsolutePath().toString());
            return Native.load(tempLib.toAbsolutePath().toString(), UnleashFFI.class, options);
        } catch (IOException e) {
            throw new RuntimeException("Failed to load native library", e);
        }
    }

    // There's no reliable way ask the JVM if this is running on alpine/musl
    // Since System.getProperty("os.name") just lists 'linux'
    // But! Because the JVM itself is dynamically linked against either libc or
    // musl, and we know the JVM is currently running, it must have loaded ld-musl
    // into memory in order to run on a musl system, so we can just query the list
    // of loaded libraries and check if ld-musl is in there
    private static boolean isMusl() {
        try (BufferedReader reader = new BufferedReader(new FileReader("/proc/self/maps"))) {
            String line;
            while ((line = reader.readLine()) != null) {
                if (line.contains("musl")) {
                    return true;
                }
            }
        } catch (IOException e) {
            System.err.println(
                    "Warning: Failed to read /proc/self/maps, assuming this is not a musl system: " + e.getMessage());
        }
        return false;
    }

    private static Path extractLibraryFromJar(String libName) throws IOException {
        Path tempFile = Files.createTempFile("lib", libName);
        try (InputStream in = UnleashFFI.class.getResourceAsStream("/native/" + libName);
                OutputStream out = Files.newOutputStream(tempFile)) {
            if (in == null) {
                throw new FileNotFoundException("File " + libName + " was not found inside JAR.");
            }

            byte[] buffer = new byte[1024];
            int readBytes;
            while ((readBytes = in.read(buffer)) != -1) {
                out.write(buffer, 0, readBytes);
            }
        }
        return tempFile;
    }
}
