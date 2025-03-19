package io.getunleash.engine;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import com.sun.jna.Memory;
import com.sun.jna.Pointer;
import java.io.IOException;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.stream.Stream;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class UnleashEngine {
    private static final Logger log = LoggerFactory.getLogger(UnleashEngine.class);
    private final UnleashFFI yggdrasil;
    private final Pointer enginePtr;
    private final ObjectMapper mapper;
    private final CustomStrategiesEvaluator customStrategiesEvaluator;
    private Object cleaner = setupCleaner();

    public UnleashEngine() {
        this(UnleashFFI.getInstance(), null, null);
    }

    public UnleashEngine(List<IStrategy> customStrategies) {
        this(UnleashFFI.getInstance(), customStrategies, null);
    }

    public UnleashEngine(List<IStrategy> customStrategies, IStrategy fallbackStrategy) {
        this(UnleashFFI.getInstance(), customStrategies, fallbackStrategy);
    }

    UnleashEngine(
            UnleashFFI ffi,
            List<IStrategy> customStrategies, IStrategy fallbackStrategy) {
        yggdrasil = ffi;
        this.enginePtr = yggdrasil.newEngine();
        if (cleanerIsSupported()) {
            registerWithCleaner(this, enginePtr);
        }
        this.mapper = new ObjectMapper();
        this.mapper.registerModule(new JavaTimeModule());
        this.mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
        if (customStrategies != null && !customStrategies.isEmpty()) {
            List<String> builtInStrategies = readRaw(yggdrasil.builtInStrategies(),
                    new TypeReference<List<String>>() {
                    });
            this.customStrategiesEvaluator = new CustomStrategiesEvaluator(
                    customStrategies.stream()
                            .filter(
                                    strategy -> {
                                        if (builtInStrategies.contains(
                                                strategy.getName())) {
                                            log.warn(
                                                    "Custom strategy {} is already a built-in strategy. Skipping.",
                                                    strategy.getName());
                                            return false;
                                        }
                                        return true;
                                    }),
                    fallbackStrategy);
        } else {
            this.customStrategiesEvaluator = new CustomStrategiesEvaluator(Stream.empty(), fallbackStrategy);
        }
    }

    public void takeState(String toggles) throws YggdrasilInvalidInputException, YggdrasilError {
        read(yggdrasil.takeState(this.enginePtr, toUtf8Pointer(toggles)),
                new TypeReference<YggResponse<Void>>() {
                });
        customStrategiesEvaluator.loadStrategiesFor(toggles);
    }

    public Boolean isEnabled(String name, Context context)
            throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            String strategyResults = customStrategiesEvaluator.eval(name, context);
            Boolean isEnabled = read(
                    yggdrasil.checkEnabled(this.enginePtr, toUtf8Pointer(name),
                            toUtf8Pointer(jsonContext), toUtf8Pointer(strategyResults)),
                    new TypeReference<YggResponse<Boolean>>() {
                    });
            return isEnabled;
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public VariantDef getVariant(String name, Context context)
            throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            String strategyResults = customStrategiesEvaluator.eval(name, context);
            VariantDef response = read(
                    yggdrasil.checkVariant(this.enginePtr, toUtf8Pointer(name),
                            toUtf8Pointer(jsonContext),
                            toUtf8Pointer(strategyResults)),
                    new TypeReference<YggResponse<VariantDef>>() {
                    });
            return response;
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public void countToggle(String flagName, boolean enabled) throws YggdrasilError {
        read(yggdrasil.countToggle(this.enginePtr, toUtf8Pointer(flagName), enabled),
                new TypeReference<YggResponse<Void>>() {
                });
    }

    public void countVariant(String flagName, String variantName) throws YggdrasilError {
        read(yggdrasil.countVariant(this.enginePtr, toUtf8Pointer(flagName),
                toUtf8Pointer(variantName)), new TypeReference<YggResponse<Void>>() {
                });
    }

    public MetricsBucket getMetrics() throws YggdrasilError {
        MetricsBucket response = read(yggdrasil.getMetrics(this.enginePtr),
                new TypeReference<YggResponse<MetricsBucket>>() {
                });
        return response;
    }

    public boolean shouldEmitImpressionEvent(String name) throws YggdrasilError {
        Boolean response = read(
                yggdrasil.shouldEmitImpressionEvent(this.enginePtr, toUtf8Pointer(name)),
                new TypeReference<YggResponse<Boolean>>() {
                });
        return response;
    }

    public List<FeatureDef> listKnownToggles() throws YggdrasilError {
        List<FeatureDef> response = read(
                yggdrasil.listKnownToggles(this.enginePtr),
                new TypeReference<YggResponse<List<FeatureDef>>>() {
                });
        return response;
    }

    public static String getCoreVersion() {
        Pointer versionPointer = UnleashFFI.getYggdrasilCoreVersion();
        return versionPointer.getString(0);
    }

    /**
     * Handle reading from a pointer into an YggdrasilResponse and unwrapping that
     * to an object
     */
    private <T> T read(Pointer pointer, TypeReference<YggResponse<T>> clazz) throws YggdrasilError {
        YggResponse<T> wrappedResponse = readRaw(pointer, clazz);
        if (wrappedResponse.isValid()) {
            return wrappedResponse.getValue();
        } else {
            throw new YggdrasilError(wrappedResponse.errorMessage);
        }
    }

    /** Handle reading from a pointer into a String and mapping it to an object */
    private <T> T readRaw(Pointer pointer, TypeReference<T> clazz) {
        try {
            String str = pointer.getString(0, StandardCharsets.UTF_8.toString());
            try {
                return mapper.readValue(str, clazz);
            } catch (IOException e) {
                log.error("Failed to parse response from Yggdrasil: {}", str, e);
                throw new YggdrasilParseException(str, clazz.getClass(), e);
            }
        } finally {
            yggdrasil.freeResponse(pointer);
        }
    }

    static Pointer toUtf8Pointer(String str) {
        byte[] utf8Bytes = str.getBytes(StandardCharsets.UTF_8);
        Pointer pointer = new Memory(utf8Bytes.length + 1);
        pointer.write(0, utf8Bytes, 0, utf8Bytes.length);
        pointer.setByte(utf8Bytes.length, (byte) 0);
        return pointer;
    }

    static boolean cleanerIsSupported() {
        String version = System.getProperty("java.version");
        if (version.startsWith("1.")) {
            int minorVersion = Integer.parseInt(version.substring(2, 3));
            return minorVersion > 8;
        }
        return true;
    }

    @Override
    protected void finalize() {
        if (cleanerIsSupported()) {
            return;
        }
        try {
            if (enginePtr != null) {
                yggdrasil.freeEngine(enginePtr);
            }
        } catch (Exception e) {
            System.err.println("Failed to release native resource: " + e.getMessage());
        }
    }

    private static Object setupCleaner() {
        if (!cleanerIsSupported()) {
            return null;
        }

        try {
            Class<?> cleanerClass = Class.forName("java.lang.ref.Cleaner");

            Method createMethod = cleanerClass.getMethod("create");
            return createMethod.invoke(null);
        } catch (Exception e) {
            throw new RuntimeException("Failed to dynamically load Cleaner", e);
        }
    }

    private void registerWithCleaner(UnleashEngine engine, Pointer enginePtr) {
        try {
            Class<?> cleanerClass = Class.forName("java.lang.ref.Cleaner");
            Method registerMethod = cleanerClass.getMethod("register", Object.class, Runnable.class);

            // Avoid capturing the engine itself in the lambda, otherwise this prevents GC!
            UnleashFFI ffiInstance = engine.yggdrasil;
            Runnable cleanupAction = () -> {
                ffiInstance.freeEngine(enginePtr);
            };

            registerMethod.invoke(cleaner, engine, cleanupAction);
        } catch (Exception e) {
            throw new RuntimeException("Failed to dynamically load Cleaner", e);
        }
    }
}
