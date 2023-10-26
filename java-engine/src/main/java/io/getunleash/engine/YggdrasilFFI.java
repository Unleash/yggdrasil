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
    @SuppressWarnings("unused")
    private final Cleaner.Cleanable cleanable;
    private final UnleashFFI ffi;
    private final Pointer enginePtr;

    /**
     * If we want singleton we just make the constructors private
     */
    YggdrasilFFI() {
        this(System.getenv("YGGDRASIL_LIB_PATH"));
    }

    static UnleashFFI loadLibrary(String libraryPath) {
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
        return Native.load(combinedPath, UnleashFFI.class);
    }

    YggdrasilFFI(String libraryPath) {
        this(loadLibrary(libraryPath));
    }

    YggdrasilFFI(UnleashFFI ffi) {
        this.ffi = ffi;
        this.enginePtr = this.ffi.new_engine();

        // Note that the cleaning action must not refer to the object being registered. If so, the object will not become phantom reachable and the cleaning action will not be invoked automatically.
        this.cleanable = CLEANER.register(this, new YggdrasilNativeLibraryResourceCleaner(this.ffi, this.enginePtr));
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
