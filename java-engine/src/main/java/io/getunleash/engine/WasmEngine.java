package io.getunleash.engine;

import com.dylibso.chicory.runtime.ExportFunction;
import com.dylibso.chicory.runtime.HostFunction;
import com.dylibso.chicory.runtime.ImportValues;
import com.dylibso.chicory.runtime.Instance;
import com.dylibso.chicory.runtime.Memory;
import com.dylibso.chicory.wasm.types.ValueType;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonMappingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.google.flatbuffers.FlatBufferBuilder;
import com.dylibso.chicory.runtime.ByteBufferMemory;

import messaging.Context;
import messaging.Response;
import messaging.PropertyEntry;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.security.SecureRandom;
import java.util.List;
import java.util.Map;
import org.example.wasm.YggdrasilModule;

public class WasmEngine {

    private Instance instance;
    private long enginePointer;
    private ExportFunction alloc;
    private ExportFunction dealloc;
    private ExportFunction checkEnabled;
    private ExportFunction deallocResponseBuffer;
    private Memory memory;

    private static final ObjectMapper objectMapper = new ObjectMapper();

    public static String toJson(Object obj) {
        try {
            return objectMapper.writeValueAsString(obj);
        } catch (Exception e) {
            throw new RuntimeException("Failed to serialize to JSON", e);
        }
    }

    public static byte[] buildContext(String toggleName, Map<String, String> properties) {
        FlatBufferBuilder builder = new FlatBufferBuilder(1024); // is this big enough for most contexts? I want to say
                                                                 // yes but needs some benching
        int toggleNameOffset = builder.createString(toggleName);

        // Build PropertyEntry vector
        int[] props = properties.entrySet().stream()
                .map(entry -> {
                    int keyOffset = builder.createString(entry.getKey());
                    int valueOffset = builder.createString(entry.getValue());
                    return PropertyEntry.createPropertyEntry(builder, keyOffset, valueOffset);
                })
                .mapToInt(Integer::intValue)
                .toArray();

        int userIdOffset = builder.createString("some-user-called-greg");

        int propsVec = Context.createPropertiesVector(builder, props);

        Context.startContext(builder);
        Context.addToggleName(builder, toggleNameOffset);
        Context.addUserId(builder, userIdOffset);
        Context.addProperties(builder, propsVec);
        int ctx = Context.endContext(builder);

        builder.finish(ctx);
        return builder.sizedByteArray();
    }

    public WasmEngine() {
        ImportValues imports = ImportValues.builder()
                .addFunction(new HostFunction(
                        "env", "fill_random",
                        List.of(ValueType.I32, ValueType.I32),
                        List.of(ValueType.I32),
                        (Instance instance, long... args) -> {
                            int ptr = (int) args[0];
                            int len = (int) args[1];

                            if (len <= 0 || ptr < 0)
                                return new long[] { 1 };

                            byte[] randomBytes = new byte[len];
                            new SecureRandom().nextBytes(randomBytes);

                            instance.memory().write(ptr, randomBytes);

                            return new long[] { 0 };
                        }))
                .build();

        instance = Instance.builder(YggdrasilModule.load())
                .withMachineFactory(YggdrasilModule::create)
                .withImportValues(imports)
                .withMemoryFactory(limits -> new ByteBufferMemory(limits))
                .build();

        ExportFunction newEngine = instance.export("new_engine");

        this.alloc = instance.export("alloc");
        this.dealloc = instance.export("dealloc");
        this.checkEnabled = instance.export("check_enabled");
        this.deallocResponseBuffer = instance.export("dealloc_response_buffer");
        this.memory = instance.memory();

        this.enginePointer = newEngine.apply()[0];
    }

    public void takeState(String message) {

        int len = message.getBytes().length;

        int ptr = (int) alloc.apply(len)[0];
        memory.writeString(ptr, message);

        ExportFunction takeState = instance.export("take_state");

        int resultPtr = (int) takeState.apply(this.enginePointer, ptr, len)[0];

        String result = memory.readCString(resultPtr);

        System.out.println(result);
        dealloc.apply(ptr, len);
    }

    public boolean checkEnabled(String toggleName, WasmContext context)
            throws JsonMappingException, JsonProcessingException {

        byte[] contextBytes = buildContext(toggleName, context.getProperties());
        int contextPtr = (int) alloc.apply(contextBytes.length)[0];
        memory.write(contextPtr, contextBytes);

        long packed = (long) checkEnabled.apply(this.enginePointer, contextPtr,
                contextBytes.length)[0];

        // This is not sane. To receive the response we need to two things:
        // 1) a pointer
        // 2) a length so we can read the pointer value to the end but not beyond
        // However, we don't have a way to pass complex objects back to the host
        // function. We can use a pre-allocated shared buffer but we would need to have that
        // buffer size appropriately tuned for real workloads. Which requires a bunch of experimentation
        // sooooo... instead we hack this. We're using 32 bit WASM here, which means pointers are 32 bits
        // and we need a second 32 bit number to represent the length of the buffer.
        // We can pass a 64 bit number across the WASM boundary, which is really two 32 bit numbers wearing a silly hat
        int ptr = (int) (packed & 0xFFFFFFFFL);
        int len = (int) (packed >>> 32);
        byte[] bytes = instance.memory().readBytes(ptr, len);

        ByteBuffer buf = ByteBuffer.wrap(bytes);
        buf.order(ByteOrder.LITTLE_ENDIAN); // Apparently flatBuffers are little-endian

        Response response = Response.getRootAsResponse(buf);

        dealloc.apply(contextPtr, contextBytes.length);
        deallocResponseBuffer.apply(ptr, len);
        return response.enabled();
    }
}
