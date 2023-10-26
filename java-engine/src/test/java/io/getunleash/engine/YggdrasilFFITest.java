package io.getunleash.engine;

import com.sun.jna.Pointer;
import org.junit.jupiter.api.Test;
import org.mockito.Mockito;

import java.lang.ref.Cleaner;
import java.lang.ref.PhantomReference;
import java.lang.ref.Reference;
import java.lang.ref.ReferenceQueue;
import java.lang.reflect.Field;
import java.nio.file.Paths;
import java.util.Objects;
import java.util.function.Function;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.verify;

class YggdrasilFFITest {

    static final String VALID_PATH = "../target/release";
    @Test
    void testSuccessfulLibraryLoad() {
        YggdrasilFFI ffi = new YggdrasilFFI(VALID_PATH);
        assertNotNull(ffi);
    }

    @Test
    void testFailedLibraryLoad() {
        assertThrows(UnsatisfiedLinkError.class, () ->
            new YggdrasilFFI("/invalid/path")
        );
    }

    @Test
    void testEngineMethods() {
        YggdrasilFFI ffi = new YggdrasilFFI(VALID_PATH);
        Pointer state = ffi.takeState("someToggles");
        assertNotNull(state);
        ffi.freeResponse(state);
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

    @Test

    void testResourceCleanup() throws InterruptedException {
        // Create a library instance and then nullify it to make it eligible for GC.
        UnleashFFI ffiMock = Mockito.mock(UnleashFFI.class);
        @SuppressWarnings("UnusedDeclaration")
        YggdrasilFFI library = new YggdrasilFFI(ffiMock);
        Cleaner.Cleanable cleanable = YggdrasilFFI.CLEANABLES.iterator().next();


        ReferenceQueue<Object> queue = new ReferenceQueue<>();
        PhantomReference<Object> ref =
                new PhantomReference<>(cleanable, queue);
        // Only the Cleaner will have a strong
        // reference to the Cleanable

        // Check that the cleanup does not happen
        // before the reference is cleared.
        assertNull(waitForReference(queue),
                "YggdrasilFFI cleaned prematurely");

        library = null; // Remove the reference

        assertNull(waitForReference(queue),
                "After GC, cleaned");

        verify(ffiMock).free_engine(Mockito.any());
    }

    private String absoluteValidPath() {
        return Paths.get(VALID_PATH).toAbsolutePath().toString();
    }

    /**
     * Wait for a Reference to be enqueued.
     * Returns null if no reference is queued within 0.1 seconds
     */
    static Reference<?> waitForReference(ReferenceQueue<Object> queue) {
        Objects.requireNonNull(queue);
        for (int i = 10; i > 0; i--) {
            System.gc();
            try {
                var r = queue.remove(10L);
                if (r != null) {
                    return r;
                }
            } catch (InterruptedException ie) {
                // ignore, the loop will try again
            }
        };
        return null;
    }
}