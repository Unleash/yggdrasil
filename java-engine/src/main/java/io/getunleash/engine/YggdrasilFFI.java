package io.getunleash.engine;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Platform;
import com.sun.jna.Pointer;

import java.lang.ref.Cleaner;
import java.nio.file.Paths;

interface UnleashFFI extends Library {

    Pointer new_engine();

    void free_engine(Pointer ptr);

    Pointer take_state(Pointer ptr, String toggles);

    Pointer check_enabled(Pointer ptr, String name, String context);

    Pointer check_variant(Pointer ptr, String name, String context);

    void free_response(Pointer pointer);
}

class YggdrasilFFI  {
    private static final Cleaner CLEANER = Cleaner.create();
    private final UnleashFFI ffi;
    private final Pointer enginePtr;

    /**
     * If we want singleton we just make the constructors private
     */
    YggdrasilFFI() {
        this(System.getenv("YGGDRASIL_LIB_PATH"));
    }

    YggdrasilFFI(String libraryPath) {
        if (libraryPath == null) {
            libraryPath = "."; // assume it's accessible in current path
        }
        System.out.println("Loading library from "+Paths.get(libraryPath).toAbsolutePath());
        String libImpl = "libyggdrasilffi.so";
        if (Platform.isMac()) {
            libImpl = "libyggdrasilffi.dylib";
        } else if (Platform.isWindows()) {
            libImpl = "libyggdrasilffi.dll";
        }

        String combinedPath = Paths.get(libraryPath, libImpl).toString();

        this.ffi = Native.load(combinedPath, UnleashFFI.class);
        this.enginePtr = this.ffi.new_engine();
        CLEANER.register(this, new YggdrasilNativeLibraryResourceCleaner(this));
    }

    Pointer takeState(String toggles) {
        return this.ffi.take_state(this.enginePtr, toggles);
    }

    void freeResponse(Pointer response) {
        this.ffi.free_response(response);
    }

    Pointer checkEnabled(String name, String context){
        return this.ffi.check_enabled(this.enginePtr, name, context);
    }

    Pointer checkVariant(String name, String context){
        return this.ffi.check_variant(this.enginePtr, name, context);
    }

    void close() {
        this.ffi.free_engine(this.enginePtr);
    }

    private static final class YggdrasilNativeLibraryResourceCleaner implements Runnable {
        private final UnleashFFI ffi;
        private final Pointer enginePtr;

        private YggdrasilNativeLibraryResourceCleaner(YggdrasilFFI yggdrasilEmbeddedLibrary) {
            this.ffi = yggdrasilEmbeddedLibrary.ffi;
            this.enginePtr = yggdrasilEmbeddedLibrary.enginePtr;
        }

        @Override
        public void run() {
            this.ffi.free_engine(this.enginePtr);
        }
    }
}
