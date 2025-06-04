package io.getunleash.engine;

import com.dylibso.chicory.runtime.ByteBufferMemory;
import com.dylibso.chicory.runtime.ExportFunction;
import com.dylibso.chicory.runtime.HostFunction;
import com.dylibso.chicory.runtime.ImportValues;
import com.dylibso.chicory.runtime.Instance;
import com.dylibso.chicory.runtime.TrapException;
import com.dylibso.chicory.wasm.types.ValueType;
import com.google.flatbuffers.FlatBufferBuilder;
import java.net.InetAddress;
import java.net.UnknownHostException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.security.SecureRandom;
import java.time.Instant;
import java.time.ZoneOffset;
import java.time.ZonedDateTime;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.function.Function;
import java.util.stream.Stream;

import org.example.wasm.YggdrasilModule;

import messaging.BuiltInStrategies;
import messaging.ContextMessage;
import messaging.CoreVersion;
import messaging.FeatureDefs;
import messaging.MetricsResponse;
import messaging.PropertyEntry;
import messaging.Response;
import messaging.ToggleEntry;
import messaging.ToggleStats;
import messaging.Variant;
import messaging.VariantEntry;
import messaging.VariantPayload;

public class UnleashEngine {

  private static Instance instance;
  private static ExportFunction alloc;
  private static ExportFunction dealloc;
  private static ExportFunction takeState;
  private static ExportFunction checkEnabled;
  private static ExportFunction checkVariant;
  private static ExportFunction getMetrics;
  private static ExportFunction deallocResponseBuffer;
  private static ExportFunction getLogBufferPtr;
  private static ExportFunction listKnownToggles;
  private static ExportFunction getCoreVersion;
  private static ExportFunction getBuiltInStrategies;
  private static ExportFunction newEngine;
  private static Object engineLock = new Object();
  private int enginePointer;
  private final CustomStrategiesEvaluator customStrategiesEvaluator;

  static {
    List<ValueType> params = new ArrayList<>();
    params.add(ValueType.I32);
    params.add(ValueType.I32);

    List<ValueType> results = new ArrayList<>();
    results.add(ValueType.I32);

    ImportValues imports = ImportValues.builder()
        .addFunction(
            new HostFunction(
                "env",
                "fill_random",
                params,
                results,
                (Instance instance, long... args) -> {
                  int ptr = (int) args[0];
                  int len = (int) args[1];

                  if (len <= 0 || ptr < 0)
                    return new long[] { 1 };

                  byte[] randomBytes = new byte[len];
                  new SecureRandom().nextBytes(randomBytes);

                  synchronized (engineLock) {
                    instance.memory().write(ptr, randomBytes);
                  }

                  return new long[] { 0 };
                }))
        .build();

    instance = Instance.builder(YggdrasilModule.load())
        .withMachineFactory(YggdrasilModule::create)
        .withImportValues(imports)
        .withMemoryFactory(limits -> new ByteBufferMemory(limits))
        .build();

    newEngine = instance.export("new_engine");
    alloc = instance.export("local_alloc");
    dealloc = instance.export("local_dealloc");
    takeState = instance.export("take_state");
    checkEnabled = instance.export("check_enabled");
    checkVariant = instance.export("check_variant");
    getMetrics = instance.export("get_metrics");
    deallocResponseBuffer = instance.export("dealloc_response_buffer");
    getLogBufferPtr = instance.export("get_log_buffer_ptr");
    listKnownToggles = instance.export("list_known_toggles");
    getCoreVersion = instance.export("get_core_version");
    getBuiltInStrategies = instance.export("get_built_in_strategies");
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
      int propOffset = PropertyEntry.createPropertyEntry(builder, keyOffset, entry.getValue() ? 1 : 0);
      offsets.add(propOffset);
    }
    return offsets.stream().mapToInt(Integer::intValue).toArray();
  }

  private static byte[] buildMessage(
      String toggleName, Context context, Map<String, Boolean> customStrategyResults) {
    FlatBufferBuilder builder = new FlatBufferBuilder(1024);

    int toggleNameOffset = builder.createString(toggleName);

    int userIdOffset = context.getUserId() != null ? builder.createString(context.getUserId()) : 0;

    int sessionIdOffset = context.getSessionId() != null ? builder.createString(context.getSessionId()) : 0;

    int appNameOffset = context.getAppName() != null ? builder.createString(context.getAppName()) : 0;

    int remoteAddressOffset = context.getRemoteAddress() != null ? builder.createString(context.getRemoteAddress()) : 0;

    String currentTime = context.getCurrentTime() != null
        ? context.getCurrentTime()
        : java.time.Instant.now().toString();
    int currentTimeOffset = builder.createString(currentTime);

    int environmentOffset = context.getEnvironment() != null ? builder.createString(context.getEnvironment()) : 0;

    int[] propertyOffsets = buildProperties(builder, context.getProperties());
    int[] customStrategyResultsOffsets = buildCustomStrategyResults(builder, customStrategyResults);

    String runtimeHostname = getRuntimeHostname();
    int runtimeHostnameOffset = runtimeHostname != null
        ? builder.createString(runtimeHostname)
        : builder.createString(getRuntimeHostname());

    int propsVec = ContextMessage.createPropertiesVector(builder, propertyOffsets);
    int customStrategyResultsVec = ContextMessage.createCustomStrategiesResultsVector(builder,
        customStrategyResultsOffsets);

    ContextMessage.startContextMessage(builder);

    if (userIdOffset != 0)
      ContextMessage.addUserId(builder, userIdOffset);
    if (sessionIdOffset != 0)
      ContextMessage.addSessionId(builder, sessionIdOffset);
    if (appNameOffset != 0)
      ContextMessage.addAppName(builder, appNameOffset);
    if (environmentOffset != 0)
      ContextMessage.addEnvironment(builder, environmentOffset);
    if (remoteAddressOffset != 0)
      ContextMessage.addRemoteAddress(builder, remoteAddressOffset);
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

  public UnleashEngine(List<IStrategy> customStrategies) {
    this(customStrategies, null);
  }

  public UnleashEngine(List<IStrategy> customStrategies, IStrategy fallbackStrategy) {
    if (customStrategies != null && !customStrategies.isEmpty()) {
      List<String> builtInStrategies = new ArrayList<>();
      this.customStrategiesEvaluator = new CustomStrategiesEvaluator(
          customStrategies.stream(), fallbackStrategy, new HashSet<String>(builtInStrategies));
    } else {
      this.customStrategiesEvaluator = new CustomStrategiesEvaluator(Stream.empty(), fallbackStrategy,
          new HashSet<String>());
    }

    ZonedDateTime now = ZonedDateTime.now(ZoneOffset.UTC);
    synchronized (engineLock) {
      this.enginePointer = (int) newEngine.apply(now.toInstant().toEpochMilli())[0];
    }
  }

  public List<String> takeState(String clientFeatures) throws YggdrasilInvalidInputException {

    try {
      customStrategiesEvaluator.loadStrategiesFor(clientFeatures);

      byte[] messageBytes = clientFeatures.getBytes();
      int len = messageBytes.length;
      synchronized (engineLock) {
        int ptr = (int) alloc.apply(len)[0];

        instance.memory().write(ptr, messageBytes);
        takeState.apply(this.enginePointer, ptr, len);

        dealloc.apply(ptr, len);
      }

      // readLog();
    } catch (TrapException e) {
      // readLog();
      throw e;
    }
    return null;
  }

  public List<FeatureDef> listKnownToggles() {
    FeatureDefs featureDefs;
    synchronized (engineLock) {
      long packed = (long) listKnownToggles.apply(this.enginePointer)[0];
      featureDefs = derefWasmPointer(packed,
          FeatureDefs::getRootAsFeatureDefs);
    }

    List<FeatureDef> defs = new ArrayList<>(featureDefs.itemsLength());
    for (int i = 0; i < featureDefs.itemsLength(); i++) {
      FeatureDef featureDef = new FeatureDef(
          featureDefs.items(i).name(),
          featureDefs.items(i).type(),
          featureDefs.items(i).project(),
          featureDefs.items(i).enabled());
      defs.add(featureDef);
    }

    return defs;
  }

  public WasmResponse<Boolean> isEnabled(String toggleName, Context context)
      throws YggdrasilInvalidInputException {
    Map<String, Boolean> strategyResults = customStrategiesEvaluator.eval(toggleName, context);
    byte[] contextBytes = buildMessage(toggleName, context, strategyResults);

    Response response;
    synchronized (engineLock) {
      int contextPtr = (int) alloc.apply(contextBytes.length)[0];
      instance.memory().write(contextPtr, contextBytes);

      response = this.<Response>callWasmFunctionWithResponse(
          contextPtr, contextBytes.length, checkEnabled::apply,
          Response::getRootAsResponse);
    }

    if (response.error() != null) {
      String error = response.error();
      throw new YggdrasilInvalidInputException(error);
    }

    if (response.hasEnabled()) {
      return new WasmResponse<Boolean>(response.impressionData(),
          response.enabled());
    } else {
      return new WasmResponse<Boolean>(response.impressionData(), null);
    }
  }

  private void readLog() {
    int start = (int) getLogBufferPtr.apply()[0];
    String msg = instance.memory().readCString(start);
    if (msg != null && !msg.isEmpty()) {
      System.out.println("DebugLog: " + msg);
    }
  }

  public WasmResponse<VariantDef> getVariant(String toggleName, Context context)
      throws YggdrasilInvalidInputException {
    Map<String, Boolean> strategyResults = customStrategiesEvaluator.eval(toggleName, context);
    byte[] contextBytes = buildMessage(toggleName, context, strategyResults);

    Variant variant;
    synchronized (engineLock) {
      int contextPtr = (int) alloc.apply(contextBytes.length)[0];
      instance.memory().write(contextPtr, contextBytes);

      variant = this.<Variant>callWasmFunctionWithResponse(
          contextPtr, contextBytes.length, checkVariant::apply,
          Variant::getRootAsVariant);

    }
    if (variant.name() != null) {
      Payload payload = null;

      VariantPayload variantPayload = variant.payload();

      if (variantPayload != null) {
        payload = new Payload();
        payload.setType(variant.payload().payloadType());
        payload.setValue(variant.payload().value());
      }

      if (variant.error() != null) {
        String error = variant.error();
        throw new YggdrasilInvalidInputException(error);
      }

      return new WasmResponse<VariantDef>(
          variant.impressionData(),
          new VariantDef(variant.name(), payload, variant.enabled(),
              variant.featureEnabled()));
    } else {
      return new WasmResponse<VariantDef>(false, null);
    }
  }

  public MetricsBucket getMetrics() {
    ZonedDateTime now = ZonedDateTime.now(ZoneOffset.UTC);

    MetricsResponse response;
    synchronized (engineLock) {
      long packed = (long) getMetrics.apply(this.enginePointer,
          now.toInstant().toEpochMilli())[0];
      response = derefWasmPointer(packed,
          MetricsResponse::getRootAsMetricsResponse);
    }
    if (response.togglesVector() == null) {
      return null;
    }

    Map<String, FeatureCount> toggles = new HashMap<>();
    for (int i = 0; i < response.togglesLength(); i++) {
      ToggleEntry toggleEntry = response.toggles(i);
      ToggleStats stats = toggleEntry.value();

      Map<String, Long> variants = new HashMap<>();
      for (int j = 0; j < stats.variantsLength(); j++) {
        VariantEntry variant = stats.variants(j);
        variants.put(variant.key(), variant.value());
      }
      FeatureCount featureCount = new FeatureCount(stats.yes(), stats.no(),
          variants);

      toggles.put(toggleEntry.key(), featureCount);
    }

    Instant startInstant = Instant.ofEpochMilli(response.start());
    Instant stopInstant = Instant.ofEpochMilli(response.stop());

    return new MetricsBucket(startInstant, stopInstant, toggles);
  }

  public static String getCoreVersion() {
    CoreVersion version;
    synchronized (engineLock) {
      long packed = (long) getCoreVersion.apply()[0];
      version = derefWasmPointer(packed,
          CoreVersion::getRootAsCoreVersion);
    }
    return version.version();
  }

  public static List<String> getBuiltInStrategies() {
    BuiltInStrategies builtInStrategiesMessage;
    synchronized (engineLock) {
      long packed = (long) getBuiltInStrategies.apply()[0];
      builtInStrategiesMessage = derefWasmPointer(packed, BuiltInStrategies::getRootAsBuiltInStrategies);
    }

    List<String> builtInStrategies = new ArrayList<>(builtInStrategiesMessage.valuesLength());
    for (int i = 0; i < builtInStrategiesMessage.valuesLength(); i++) {
      String strategyName = builtInStrategiesMessage.values(i);
      builtInStrategies.add(strategyName);
    }

    return builtInStrategies;
  }

  private static <T> T derefWasmPointer(long packed, Function<ByteBuffer, T> decoder) {
    // Warning: This is not thread safe, it should be called as part of a
    // synchronized block
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
      TriFunction<Integer, Integer, Integer, long[]> nativeCall,
      Function<ByteBuffer, T> decoder) {
    // Warning: This is not thread safe, it should be called as part of a
    // synchronized block
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
