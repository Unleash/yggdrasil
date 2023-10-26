package io.getunleash.engine;

import com.sun.jna.Pointer;
import org.junit.jupiter.api.Test;

import java.nio.file.Paths;

import static org.junit.jupiter.api.Assertions.*;

class YggdrasilFFITest {

    static final String VALID_PATH = "../target/release";
    @Test
    void testSuccessfulLibraryLoad() {
        try (YggdrasilFFI ffi = new YggdrasilFFI(VALID_PATH)) {
            assertNotNull(ffi);
        }
    }

    @Test
    void testFailedLibraryLoad() {
        assertThrows(UnsatisfiedLinkError.class, () ->
            new YggdrasilFFI("/invalid/path")
        );
    }

    @Test
    void testClose() {
        YggdrasilFFI ffi = new YggdrasilFFI(VALID_PATH);
        ffi.close();
        // maybe have some mechanism to ensure free_engine was called.
    }

    @Test
    void testEngineMethods() {
        try (YggdrasilFFI ffi = new YggdrasilFFI(VALID_PATH)) {
            Pointer state = ffi.takeState("someToggles");
            assertNotNull(state);
            // add more methods and assertions as needed.
        }
    }

    @Test
    void testLibraryPathVariations() {
        assertDoesNotThrow(() -> {
            new YggdrasilFFI(absoluteValidPath());
        });

        assertThrows(UnsatisfiedLinkError.class, () -> {
            new YggdrasilFFI("/non/existent/path");
        });
    }

    private String absoluteValidPath() {
        return Paths.get(VALID_PATH).toAbsolutePath().toString();
    }
}