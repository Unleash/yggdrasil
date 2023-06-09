/*
 * This Java source file was generated by the Gradle 'init' task.
 */
package unleash.engine;

import com.sun.jna.*;
import com.sun.jna.ptr.*;
import java.util.List;
import java.util.Arrays;

public interface UnleashEngine extends Library {
    UnleashEngine INSTANCE = (UnleashEngine) Native.loadLibrary(
        "/home/simon/dev/yggdrasil/target/release/libyggdrasilffi.so", UnleashEngine.class);

    Pointer engine_new();
    void engine_free(Pointer ptr);
    String engine_take_state(Pointer ptr, String toggles);
    boolean engine_is_enabled(Pointer ptr, String input, FFIContext.ByValue context);
    Pointer engine_get_variant(Pointer ptr, String name, FFIContext.ByValue context);
    void engine_free_variant_def(Pointer variant_def_ptr);

    public static class Payload extends Structure {
        public static class ByValue extends Payload implements Structure.ByValue {}

        public static class ByReference extends Payload implements Structure.ByReference {}

        public Pointer payload_type;
        public Pointer value;

        @Override
        protected List<String> getFieldOrder() {
            return Arrays.asList("payload_type", "value");
        }
    }

    public static class VariantDef extends Structure {
        public static class ByValue extends VariantDef implements Structure.ByValue {}

        public static class ByReference extends VariantDef implements Structure.ByReference {}

        public Pointer name;
        public Pointer payload;
        public boolean enabled;

        @Override
        protected List<String> getFieldOrder() {
            return Arrays.asList("name", "payload", "enabled");
        }
    }

    public static class FFIContext extends Structure {
        public static class ByValue extends FFIContext implements Structure.ByValue {}

        public static class ByReference extends FFIContext implements Structure.ByReference {}

        public Pointer user_id;
        public Pointer session_id;
        public Pointer environment;
        public Pointer app_name;
        public Pointer current_time;
        public Pointer remote_address;
        public Pointer properties_keys;
        public Pointer properties_values;
        public long properties_len;
        public Pointer toggle_name;

        @Override
        protected List<String> getFieldOrder() {
            return Arrays.asList("user_id", "session_id", "environment", "app_name", "current_time", "remote_address", "properties_keys", "properties_values", "properties_len", "toggle_name");
        }
    }
}