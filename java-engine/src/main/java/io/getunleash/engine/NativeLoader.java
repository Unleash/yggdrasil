package io.getunleash.engine;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Collections;
import java.lang.System;

interface UnleashFFI extends Library {

    Pointer newEngine();

    void freeEngine(Pointer ptr);

    Pointer takeState(Pointer ptr, String toggles);

    Pointer checkEnabled(Pointer ptr, String name, String context, String customStrategyResults);

    Pointer checkVariant(Pointer ptr, String name, String context, String customStrategyResults);

    void countToggle(Pointer ptr, String name, boolean enabled);

    void countVariant(Pointer ptr, String name, String variantName);

    Pointer getMetrics(Pointer ptr);

    Pointer shouldEmitImpressionEvent(Pointer ptr, String name);

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
            if (arch.contains("musl")) {
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

        try {
            // Extract and load the native library from the JAR
            Path tempLib = extractLibraryFromJar(libName);
            System.load(tempLib.toAbsolutePath().toString());
            return Native.load(tempLib.toAbsolutePath().toString(), UnleashFFI.class,
                    Collections.singletonMap(Library.OPTION_FUNCTION_MAPPER, new CamelToSnakeMapper()));
        } catch (IOException e) {
            throw new RuntimeException("Failed to load native library", e);
        }
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
