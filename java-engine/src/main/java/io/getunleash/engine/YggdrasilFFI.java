package io.getunleash.engine;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Platform;
import com.sun.jna.Pointer;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import java.lang.System;

interface UnleashFFI extends Library {

    Pointer new_engine();

    void free_engine(Pointer ptr);

    Pointer take_state(Pointer ptr, String toggles);

    Pointer check_enabled(Pointer ptr, String name, String context, String customStrategyResults);

    Pointer check_variant(Pointer ptr, String name, String context, String customStrategyResults);

    void count_toggle(Pointer ptr, String name, boolean enabled);

    void count_variant(Pointer ptr, String name, String variantName);

    Pointer get_metrics(Pointer ptr);

    Pointer should_emit_impression_event(Pointer ptr, String name);

    Pointer built_in_strategies();

    void free_response(Pointer pointer);
}

class YggdrasilFFI {
    private static final Logger LOG = LoggerFactory.getLogger(YggdrasilFFI.class);

    private final UnleashFFI ffi;
    private final Pointer enginePtr;

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
            if (arch.equals("x86_64")) {
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
            return Native.load(tempLib.toAbsolutePath().toString(), UnleashFFI.class);
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

    YggdrasilFFI() {
        this(loadLibrary());
    }

    YggdrasilFFI(UnleashFFI ffi) {
        this.ffi = ffi;
        this.enginePtr = this.ffi.new_engine();
    }

    @Override
    protected void finalize() {
        new YggdrasilNativeLibraryResourceCleaner(this.ffi, this.enginePtr).run();
    }

    Pointer takeState(String toggles) {
        return this.ffi.take_state(this.enginePtr, toggles);
    }

    void freeResponse(Pointer response) {
        this.ffi.free_response(response);
    }

    Pointer checkEnabled(String name, String context, String customStrategyResults) {
        return this.ffi.check_enabled(this.enginePtr, name, context, customStrategyResults);
    }

    Pointer checkVariant(String name, String context, String customStrategyResults) {
        return this.ffi.check_variant(this.enginePtr, name, context, customStrategyResults);
    }

    void countToggle(String flagName, boolean enabled) {
        this.ffi.count_toggle(this.enginePtr, flagName, enabled);
    }

    void countVariant(String flagName, String variantName) {
        this.ffi.count_variant(this.enginePtr, flagName, variantName);
    }

    Pointer getMetrics() {
        return this.ffi.get_metrics(this.enginePtr);
    }

    Pointer builtInStrategies() {
        return this.ffi.built_in_strategies();
    }

    Pointer shouldEmitImpressionEvent(String name) {
        return this.ffi.should_emit_impression_event(this.enginePtr, name);
    }

    private static final class YggdrasilNativeLibraryResourceCleaner implements Runnable {
        private final UnleashFFI ffi;
        private final Pointer enginePtr;

        private YggdrasilNativeLibraryResourceCleaner(UnleashFFI ffi, Pointer enginePtr) {
            this.ffi = ffi;
            this.enginePtr = enginePtr;
        }

        @Override
        public void run() {
            // All exceptions thrown by the cleaning action are ignored
            this.ffi.free_engine(this.enginePtr);
        }
    }
}
