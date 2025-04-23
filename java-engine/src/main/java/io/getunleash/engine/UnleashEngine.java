package io.getunleash.engine;

import com.dylibso.chicory.runtime.ByteBufferMemory;
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
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.security.SecureRandom;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import messaging.ContextMessage;
import messaging.PropertyEntry;
import messaging.Response;
import org.example.wasm.YggdrasilModule;

public class UnleashEngine {

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

  public static byte[] buildMessage(String toggleName, Context context) {
    FlatBufferBuilder builder = new FlatBufferBuilder(1024);

    int toggleNameOffset = builder.createString(toggleName);

    int userIdOffset = context.getUserId() != null ? builder.createString(context.getUserId()) : 0;

    int sessionIdOffset =
        context.getSessionId() != null ? builder.createString(context.getSessionId()) : 0;

    int appNameOffset =
        context.getAppName() != null ? builder.createString(context.getAppName()) : 0;

    String currentTime =
        context.getCurrentTime() != null
            ? context.getCurrentTime()
            : java.time.Instant.now().toString();
    int currentTimeOffset = builder.createString(currentTime);

    int environmentOffset =
        context.getEnvironment() != null ? builder.createString(context.getEnvironment()) : 0;

    List<Map.Entry<String, String>> entries = new ArrayList<>(context.properties.entrySet());
    int[] propertyOffsets = new int[entries.size()];
    for (int i = 0; i < entries.size(); i++) {
      Map.Entry<String, String> entry = entries.get(i);
      int keyOffset = builder.createString(entry.getKey());
      int valueOffset = builder.createString(entry.getValue());
      propertyOffsets[i] = PropertyEntry.createPropertyEntry(builder, keyOffset, valueOffset);
    }

    ContextMessage.startContextMessage(builder);

    if (userIdOffset != 0) ContextMessage.addUserId(builder, userIdOffset);
    if (sessionIdOffset != 0) ContextMessage.addSessionId(builder, sessionIdOffset);
    if (appNameOffset != 0) ContextMessage.addAppName(builder, appNameOffset);
    if (environmentOffset != 0) ContextMessage.addEnvironment(builder, environmentOffset);

    ContextMessage.addCurrentTime(builder, currentTimeOffset);
    ContextMessage.addToggleName(builder, toggleNameOffset);

    if (propertyOffsets.length > 0) {
      int propsVec = ContextMessage.createPropertiesVector(builder, propertyOffsets);
      ContextMessage.addProperties(builder, propsVec);
    }

    int ctx = ContextMessage.endContextMessage(builder);
    builder.finish(ctx);
    return builder.sizedByteArray();
  }

  public UnleashEngine() {
    ImportValues imports =
        ImportValues.builder()
            .addFunction(
                new HostFunction(
                    "env",
                    "fill_random",
                    List.of(ValueType.I32, ValueType.I32),
                    List.of(ValueType.I32),
                    (Instance instance, long... args) -> {
                      int ptr = (int) args[0];
                      int len = (int) args[1];

                      if (len <= 0 || ptr < 0) return new long[] {1};

                      byte[] randomBytes = new byte[len];
                      new SecureRandom().nextBytes(randomBytes);

                      instance.memory().write(ptr, randomBytes);

                      return new long[] {0};
                    }))
            .build();

    instance =
        Instance.builder(YggdrasilModule.load())
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

  public boolean isEnabled(String toggleName, Context context)
      throws JsonMappingException, JsonProcessingException {

    byte[] contextBytes = buildMessage(toggleName, context);
    int contextPtr = (int) alloc.apply(contextBytes.length)[0];
    memory.write(contextPtr, contextBytes);

    long packed = (long) checkEnabled.apply(this.enginePointer, contextPtr, contextBytes.length)[0];

    // This is not sane. To receive the response we need to two things:
    // 1) a pointer
    // 2) a length so we can read the pointer value to the end but not beyond
    // However, we don't have a way to pass complex objects back to the host
    // function. We can use a pre-allocated shared buffer but we would need to have that
    // buffer size appropriately tuned for real workloads. Which requires a bunch of experimentation
    // sooooo... instead we hack this. We're using 32 bit WASM here, which means pointers are 32
    // bits
    // and we need a second 32 bit number to represent the length of the buffer.
    // We can pass a 64 bit number across the WASM boundary, which is really two 32 bit numbers
    // wearing a silly hat
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
