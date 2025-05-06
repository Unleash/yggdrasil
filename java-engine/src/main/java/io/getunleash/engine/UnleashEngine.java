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
import java.net.InetAddress;
import java.net.UnknownHostException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.security.SecureRandom;
import java.time.ZoneOffset;
import java.time.ZonedDateTime;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.function.Function;
import java.util.stream.Stream;
import messaging.ContextMessage;
import messaging.FeatureDefs;
import messaging.MetricsBucket;
import messaging.PropertyEntry;
import messaging.Response;
import messaging.Variant;
import messaging.VariantPayload;
import org.example.wasm.YggdrasilModule;

public class UnleashEngine {

  private Instance instance;
  private long enginePointer;
  private ExportFunction alloc;
  private ExportFunction dealloc;
  private ExportFunction checkEnabled;
  private ExportFunction checkVariant;
  private ExportFunction getMetrics;
  private ExportFunction deallocResponseBuffer;
  private ExportFunction getLogBufferPtr;
  private ExportFunction listKnownToggles;
  private Memory memory;

  private final CustomStrategiesEvaluator customStrategiesEvaluator;

  private static final ObjectMapper objectMapper = new ObjectMapper();

  public static String toJson(Object obj) {
    try {
      return objectMapper.writeValueAsString(obj);
    } catch (Exception e) {
      throw new RuntimeException("Failed to serialize to JSON", e);
    }
  }

  private static String getRuntimeHostname() {
    String hostname = System.getProperty("hostname");
    if (hostname == null) {
      try {
        hostname = InetAddress.getLocalHost().getHostName();
      } catch (UnknownHostException e) {
        hostname = "undefined";
      }
    }
    return hostname;
  }

  private static int[] buildProperties(FlatBufferBuilder builder, Map<String, String> properties) {
    List<Map.Entry<String, String>> entries = new ArrayList<>(properties.entrySet());
    List<Integer> offsets = new ArrayList<>();
    for (Map.Entry<String, String> entry : entries) {
      if (entry.getValue() == null) {
        continue;
      }
      int keyOffset = builder.createString(entry.getKey());
      int valueOffset = builder.createString(entry.getValue());
      int propOffset = PropertyEntry.createPropertyEntry(builder, keyOffset, valueOffset);
      offsets.add(propOffset);
    }
    return offsets.stream().mapToInt(Integer::intValue).toArray();
  }

  private static int[] buildCustomStrategyResults(
      FlatBufferBuilder builder, Map<String, Boolean> results) {
    List<Map.Entry<String, Boolean>> entries = new ArrayList<>(results.entrySet());
    List<Integer> offsets = new ArrayList<>();
    for (Map.Entry<String, Boolean> entry : entries) {
      if (entry.getValue() == null) {
        continue;
      }
      int keyOffset = builder.createString(entry.getKey());
      int valueOffset = builder.createString(entry.getValue().toString());
      int propOffset = PropertyEntry.createPropertyEntry(builder, keyOffset, valueOffset);
      offsets.add(propOffset);
    }
    return offsets.stream().mapToInt(Integer::intValue).toArray();
  }

  private static byte[] buildMessage(
      String toggleName, Context context, Map<String, Boolean> customStrategyResults) {
    FlatBufferBuilder builder = new FlatBufferBuilder(1024);

    int toggleNameOffset = builder.createString(toggleName);

    int userIdOffset = context.getUserId() != null ? builder.createString(context.getUserId()) : 0;

    int sessionIdOffset =
        context.getSessionId() != null ? builder.createString(context.getSessionId()) : 0;

    int appNameOffset =
        context.getAppName() != null ? builder.createString(context.getAppName()) : 0;

    int remoteAddressOffset =
        context.getRemoteAddress() != null ? builder.createString(context.getRemoteAddress()) : 0;

    String currentTime =
        context.getCurrentTime() != null
            ? context.getCurrentTime()
            : java.time.Instant.now().toString();
    int currentTimeOffset = builder.createString(currentTime);

    int environmentOffset =
        context.getEnvironment() != null ? builder.createString(context.getEnvironment()) : 0;

    int[] propertyOffsets = buildProperties(builder, context.properties);
    int[] customStrategyResultsOffsets = buildCustomStrategyResults(builder, customStrategyResults);

    String runtimeHostname = getRuntimeHostname();
    int runtimeHostnameOffset =
        runtimeHostname != null
            ? builder.createString(runtimeHostname)
            : builder.createString(getRuntimeHostname());

    int propsVec = ContextMessage.createPropertiesVector(builder, propertyOffsets);
    int customStrategyResultsVec =
        ContextMessage.createCustomStrategiesResultsVector(builder, customStrategyResultsOffsets);

    ContextMessage.startContextMessage(builder);

    if (userIdOffset != 0) ContextMessage.addUserId(builder, userIdOffset);
    if (sessionIdOffset != 0) ContextMessage.addSessionId(builder, sessionIdOffset);
    if (appNameOffset != 0) ContextMessage.addAppName(builder, appNameOffset);
    if (environmentOffset != 0) ContextMessage.addEnvironment(builder, environmentOffset);
    if (remoteAddressOffset != 0) ContextMessage.addRemoteAddress(builder, remoteAddressOffset);
    if (runtimeHostnameOffset != 0)
      ContextMessage.addRuntimeHostname(builder, runtimeHostnameOffset);

    ContextMessage.addCurrentTime(builder, currentTimeOffset);
    ContextMessage.addToggleName(builder, toggleNameOffset);

    if (propertyOffsets.length > 0) {
      ContextMessage.addProperties(builder, propsVec);
    }

    if (customStrategyResultsOffsets.length > 0) {
      ContextMessage.addCustomStrategiesResults(builder, customStrategyResultsVec);
    }

    int ctx = ContextMessage.endContextMessage(builder);
    builder.finish(ctx);
    return builder.sizedByteArray();
  }

  public UnleashEngine() {
    this(null, null);
  }

  public UnleashEngine(List<IStrategy> customStrategies, IStrategy fallbackStrategy) {
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

    if (customStrategies != null && !customStrategies.isEmpty()) {
      List<String> builtInStrategies = new ArrayList<>();
      this.customStrategiesEvaluator =
          new CustomStrategiesEvaluator(
              customStrategies.stream(), fallbackStrategy, new HashSet<String>(builtInStrategies));
    } else {
      this.customStrategiesEvaluator =
          new CustomStrategiesEvaluator(Stream.empty(), fallbackStrategy, new HashSet<String>());
    }

    this.alloc = instance.export("alloc");
    this.dealloc = instance.export("dealloc");
    this.checkEnabled = instance.export("check_enabled");
    this.checkVariant = instance.export("check_variant");
    this.getMetrics = instance.export("get_metrics");
    this.deallocResponseBuffer = instance.export("dealloc_response_buffer");
    this.getLogBufferPtr = instance.export("get_log_buffer_ptr");
    this.listKnownToggles = instance.export("list_known_toggles");
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

  public List<FeatureDef> listKnownToggles() throws Exception {
    long packed = (long) listKnownToggles.apply(this.enginePointer)[0];
    FeatureDefs featureDefs = derefWasmPointer(packed, FeatureDefs::getRootAsFeatureDefs);

    List<FeatureDef> defs = new ArrayList<>(featureDefs.itemsLength());
    for (int i = 0; i < featureDefs.itemsLength(); i++) {
      FeatureDef featureDef =
          new FeatureDef(
              featureDefs.items(i).name(),
              featureDefs.items(i).type(),
              featureDefs.items(i).project(),
              featureDefs.items(i).enabled());
      defs.add(featureDef);
    }

    return defs;
  }

  public Boolean isEnabled(String toggleName, Context context)
      throws JsonMappingException, JsonProcessingException {

    Map<String, Boolean> strategyResults = customStrategiesEvaluator.eval(toggleName, context);
    byte[] contextBytes = buildMessage(toggleName, context, strategyResults);
    int contextPtr = (int) alloc.apply(contextBytes.length)[0];
    memory.write(contextPtr, contextBytes);

    Response response =
        this.<Response>callWasmFunctionWithResponse(
            contextPtr, contextBytes.length, checkEnabled::apply, Response::getRootAsResponse);

    if (response.hasEnabled()) {
      return response.enabled();
    }
    return null;
  }

  private void readLog() {
    int start = (int) this.getLogBufferPtr.apply()[0];
    String msg = memory.readCString(start);
    if (msg != null && !msg.isEmpty()) {
      System.out.println("DebugLog: " + msg);
    }
  }

  public VariantDef getVariant(String toggleName, Context context)
      throws JsonMappingException, JsonProcessingException {
    Map<String, Boolean> strategyResults = customStrategiesEvaluator.eval(toggleName, context);
    byte[] contextBytes = buildMessage(toggleName, context, strategyResults);
    int contextPtr = (int) alloc.apply(contextBytes.length)[0];
    memory.write(contextPtr, contextBytes);

    Variant variant =
        this.<Variant>callWasmFunctionWithResponse(
            contextPtr, contextBytes.length, checkVariant::apply, Variant::getRootAsVariant);

    if (variant.name() != null) {
      Payload payload = null;

      VariantPayload variantPayload = variant.payload();

      if (variantPayload != null) {
        payload = new Payload();
        payload.setType(variant.payload().payloadType());
        payload.setValue(variant.payload().value());
      }

      return new VariantDef(variant.name(), payload, variant.enabled(), variant.featureEnabled());
    } else {
      return null;
    }
  }

  public MetricsBucket getMetrics() throws JsonMappingException, JsonProcessingException {
    ZonedDateTime now = ZonedDateTime.now(ZoneOffset.UTC);

    long packed = (long) getMetrics.apply(this.enginePointer, now.toInstant().toEpochMilli())[0];
    MetricsBucket bucket = derefWasmPointer(packed, MetricsBucket::getRootAsMetricsBucket);

    if (bucket.togglesVector() == null) {
      return null;
    }
    return bucket;
  }

  private <T> T derefWasmPointer(long packed, Function<ByteBuffer, T> decoder) {
    // This is not sane. To receive the response we need to two things:
    // 1) a pointer
    // 2) a length so we can read the pointer value to the end but not beyond
    //
    // However, we don't have a way to pass complex objects back to the host
    // function. We can use a pre-allocated shared buffer but we would need to have
    // that buffer size appropriately tuned for real workloads.
    // Which requires a bunch of experimentation sooooo...
    // instead we hack this. We're using 32 bit WASM here, which means
    // pointers are 32 bits and we need a second 32 bit number to represent the
    // length of the buffer. We can pass a 64 bit number across the WASM boundary,
    // which is really two 32 bit numbers wearing a silly hat

    int ptr = (int) (packed & 0xFFFFFFFFL);
    int len = (int) (packed >>> 32);

    byte[] bytes = instance.memory().readBytes(ptr, len);

    ByteBuffer buf = ByteBuffer.wrap(bytes);
    buf.order(ByteOrder.LITTLE_ENDIAN);

    T response = decoder.apply(buf);
    deallocResponseBuffer.apply(ptr, len);
    return response;
  }

  private <T> T callWasmFunctionWithResponse(
      int messagePtr,
      int messageLen,
      TriFunction<Long, Integer, Integer, long[]> nativeCall,
      Function<ByteBuffer, T> decoder) {
    long packed = nativeCall.apply(this.enginePointer, messagePtr, messageLen)[0];
    T response = derefWasmPointer(packed, decoder);

    dealloc.apply(messagePtr, messageLen);

    readLog();

    return response;
  }

  @FunctionalInterface
  public interface TriFunction<A, B, C, R> {
    R apply(A a, B b, C c);
  }
}
