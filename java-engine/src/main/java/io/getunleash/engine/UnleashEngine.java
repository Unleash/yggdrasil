package io.getunleash.engine;

import com.dylibso.chicory.runtime.TrapException;
import com.google.flatbuffers.FlatBufferBuilder;
import java.lang.ref.Cleaner;
import java.net.InetAddress;
import java.net.UnknownHostException;
import java.nio.charset.StandardCharsets;
import java.time.Instant;
import java.time.ZoneOffset;
import java.time.ZonedDateTime;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.stream.Stream;
import messaging.BuiltInStrategies;
import messaging.ContextMessage;
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
  private final NativeInterface nativeInterface;
  private final int enginePointer;
  private final CustomStrategiesEvaluator customStrategiesEvaluator;
  private static final Cleaner cleaner = Cleaner.create();
  private final Cleaner.Cleanable cleanable;

  public UnleashEngine() {
    this(null, null, null);
  }

  public UnleashEngine(List<IStrategy> customStrategies) {
    this(customStrategies, null, null);
  }

  public UnleashEngine(List<IStrategy> customStrategies, IStrategy fallbackStrategy) {
    this(customStrategies, fallbackStrategy, null);
  }

  public UnleashEngine(
      List<IStrategy> customStrategies,
      IStrategy fallbackStrategy,
      NativeInterface nativeInterface) {
    if (customStrategies != null && !customStrategies.isEmpty()) {
      List<String> builtInStrategies = getBuiltInStrategies();
      this.customStrategiesEvaluator =
          new CustomStrategiesEvaluator(
              customStrategies.stream(), fallbackStrategy, new HashSet<String>(builtInStrategies));
    } else {
      this.customStrategiesEvaluator =
          new CustomStrategiesEvaluator(Stream.empty(), fallbackStrategy, new HashSet<String>());
    }

    if (nativeInterface != null) {
      this.nativeInterface = nativeInterface;
    } else {
      this.nativeInterface = new WasmInterface();
    }

    Instant now = Instant.now();
    final int enginePtr = this.nativeInterface.newEngine(now.toEpochMilli());
    if (enginePtr < 0) {
      throw new IllegalStateException(
          "Failed to create Unleash engine (invalid pointer): " + enginePtr);
    }
    this.enginePointer = enginePtr;

    NativeInterface wasmHook = this.nativeInterface;

    cleanable =
        cleaner.register(
            this,
            () -> {
              wasmHook.freeEngine(enginePtr);
            });
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
      int propOffset =
          PropertyEntry.createPropertyEntry(builder, keyOffset, entry.getValue() ? 1 : 0);
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

    int[] propertyOffsets = buildProperties(builder, context.getProperties());
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

  public List<String> takeState(String clientFeatures) throws YggdrasilInvalidInputException {
    try {
      customStrategiesEvaluator.loadStrategiesFor(clientFeatures);
      byte[] messageBytes = clientFeatures.getBytes(StandardCharsets.UTF_8);
      nativeInterface.takeState(this.enginePointer, messageBytes);
    } catch (TrapException e) {
      throw e;
    }
    return null;
  }

  public List<FeatureDef> listKnownToggles() {
    FeatureDefs featureDefs = nativeInterface.listKnownToggles(this.enginePointer);

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

  public WasmResponse<Boolean> isEnabled(String toggleName, Context context)
      throws YggdrasilInvalidInputException {
    Map<String, Boolean> strategyResults = customStrategiesEvaluator.eval(toggleName, context);
    byte[] contextBytes = buildMessage(toggleName, context, strategyResults);

    Response response = nativeInterface.checkEnabled(enginePointer, contextBytes);

    if (response.error() != null) {
      String error = response.error();
      throw new YggdrasilInvalidInputException(error);
    }

    if (response.hasEnabled()) {
      return new WasmResponse<Boolean>(response.impressionData(), response.enabled());
    } else {
      return new WasmResponse<Boolean>(response.impressionData(), null);
    }
  }

  public WasmResponse<VariantDef> getVariant(String toggleName, Context context)
      throws YggdrasilInvalidInputException {
    Map<String, Boolean> strategyResults = customStrategiesEvaluator.eval(toggleName, context);
    byte[] contextBytes = buildMessage(toggleName, context, strategyResults);

    Variant variant = nativeInterface.checkVariant(enginePointer, contextBytes);
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
          new VariantDef(variant.name(), payload, variant.enabled(), variant.featureEnabled()));
    } else {
      return new WasmResponse<VariantDef>(false, null);
    }
  }

  public MetricsBucket getMetrics() {
    ZonedDateTime now = ZonedDateTime.now(ZoneOffset.UTC);

    MetricsResponse response = nativeInterface.getMetrics(this.enginePointer, now);
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
      FeatureCount featureCount = new FeatureCount(stats.yes(), stats.no(), variants);

      toggles.put(toggleEntry.key(), featureCount);
    }

    Instant startInstant = Instant.ofEpochMilli(response.start());
    Instant stopInstant = Instant.ofEpochMilli(response.stop());

    return new MetricsBucket(startInstant, stopInstant, toggles);
  }

  // The following two methods break our abstraction a little by calling the
  // WasmInterface directly. rather than through the nativeInterface. However,
  // we really, really want them to be accessible without having to instantiate
  // an UnleashEngine and our interface abstraction here is primarily for testing
  public static String getCoreVersion() {
    return WasmInterface.getCoreVersion();
  }

  public static List<String> getBuiltInStrategies() {
    BuiltInStrategies builtInStrategiesMessage = WasmInterface.getBuiltInStrategies();
    List<String> builtInStrategies = new ArrayList<>(builtInStrategiesMessage.valuesLength());
    for (int i = 0; i < builtInStrategiesMessage.valuesLength(); i++) {
      String strategyName = builtInStrategiesMessage.values(i);
      builtInStrategies.add(strategyName);
    }

    return builtInStrategies;
  }
}
