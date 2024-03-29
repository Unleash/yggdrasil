package io.getunleash.engine;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.verify;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.jna.Pointer;
import java.lang.ref.Reference;
import java.lang.ref.ReferenceQueue;
import java.util.List;
import java.util.Objects;
import org.junit.jupiter.api.Test;
import org.mockito.Mockito;

class YggdrasilFFITest {

    @Test
    void testSuccessfulLibraryLoad() {
        YggdrasilFFI ffi = new YggdrasilFFI();
        assertNotNull(ffi);
    }

    // TODO: we need this?
    // @Test
    // void testFailedLibraryLoad() {
    //     assertThrows(UnsatisfiedLinkError.class, () -> new YggdrasilFFI("/invalid/path"));
    // }

    @Test
    void testEngineMethods() {
        YggdrasilFFI ffi = new YggdrasilFFI();
        Pointer state = ffi.takeState("someToggles");
        assertNotNull(state);
        ffi.freeResponse(state);
    }

    @Test
    void testCustomStrategies() throws JsonProcessingException {
        YggdrasilFFI ffi = new YggdrasilFFI();
        Pointer ptr = ffi.builtInStrategies();
        String content = ptr.getString(0, "UTF-8");
        ffi.freeResponse(ptr);
        List<String> strategies =
                new ObjectMapper().readValue(content, new TypeReference<List<String>>() {});
        assertNotNull(strategies);
        assertFalse(strategies.isEmpty());
        assertTrue(strategies.contains("default"));
        assertTrue(strategies.contains("gradualRolloutRandom"));
    }

    @Test
    void testLibraryPathVariations() {
        assertDoesNotThrow(
                () -> {
                    new YggdrasilFFI();
                });

        // TODO: We need this?
        // assertThrows(
        //         UnsatisfiedLinkError.class,
        //         () -> {
        //             new YggdrasilFFI("/non/existent/path");
        //         });
    }

    @Test
    void testResourceCleanup() {
        // Create a library instance and then nullify it to make it eligible for GC.
        UnleashFFI ffiMock = Mockito.mock(UnleashFFI.class);
        @SuppressWarnings("UnusedDeclaration")
        YggdrasilFFI library = new YggdrasilFFI(ffiMock);

        ReferenceQueue<Object> queue = new ReferenceQueue<>();
        // Only the Cleaner will have a strong
        // reference to the Cleanable

        // Check that the cleanup does not happen
        // before the reference is cleared.
        assertNull(waitForReference(queue), "YggdrasilFFI cleaned prematurely");

        library = null; // Remove the reference

        assertNull(waitForReference(queue), "After GC, cleaned");

        verify(ffiMock).free_engine(Mockito.any());
    }

    /**
     * Wait for a Reference to be enqueued. Returns null if no reference is queued within 0.1
     * seconds
     */
    static Reference<?> waitForReference(ReferenceQueue<Object> queue) {
        Objects.requireNonNull(queue);
        for (int i = 10; i > 0; i--) {
            System.gc();
            try {
                Reference<?> r = queue.remove(10L);
                if (r != null) {
                    return r;
                }
            } catch (InterruptedException ie) {
                // ignore, the loop will try again
            }
        }
        ;
        return null;
    }
}
