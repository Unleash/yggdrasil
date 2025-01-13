package io.getunleash.engine;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import com.sun.jna.Pointer;
import java.io.IOException;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.stream.Stream;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class UnleashEngine {
    private static final String EMPTY_STRATEGY_RESULTS = "{}";
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
            List<String> builtInStrategies = read(yggdrasil.builtInStrategies(),
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

    public void takeState(String toggles) throws YggdrasilInvalidInputException {
        YggResponse<Void> response = read(yggdrasil.takeState(this.enginePtr, toggles),
                new TypeReference<YggResponse<Void>>() {
                });
        if (!response.isValid()) {
            throw new YggdrasilInvalidInputException(toggles);
        }

        customStrategiesEvaluator.loadStrategiesFor(toggles);
    }

    public Boolean isEnabled(String name, Context context)
            throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            String strategyResults = customStrategiesEvaluator.eval(name, context);
            YggResponse<Boolean> isEnabled = read(
                    yggdrasil.checkEnabled(this.enginePtr, name, jsonContext, strategyResults),
                    new TypeReference<YggResponse<Boolean>>() {
                    });
            return isEnabled.getValue();
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public VariantDef getVariant(String name, Context context)
            throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            YggResponse<VariantDef> response = read(
                    yggdrasil.checkVariant(this.enginePtr, name, jsonContext,
                            EMPTY_STRATEGY_RESULTS),
                    new TypeReference<YggResponse<VariantDef>>() {
                    });
            return response.getValue();
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public void countToggle(String flagName, boolean enabled) {
        yggdrasil.countToggle(this.enginePtr, flagName, enabled);
    }

    public void countVariant(String flagName, String variantName) {
        yggdrasil.countVariant(this.enginePtr, flagName, variantName);
    }

    public MetricsBucket getMetrics() throws YggdrasilError {
        YggResponse<MetricsBucket> response = read(yggdrasil.getMetrics(this.enginePtr),
                new TypeReference<YggResponse<MetricsBucket>>() {
                });
        return response.getValue();
    }

    public boolean shouldEmitImpressionEvent(String name) throws YggdrasilError {
        YggResponse<Boolean> response = read(
                yggdrasil.shouldEmitImpressionEvent(this.enginePtr, name),
                new TypeReference<YggResponse<Boolean>>() {
                });
        return response.getValue();
    }

    public List<FeatureDef> listKnownToggles() throws YggdrasilError {
        YggResponse<List<FeatureDef>> response = read(
                yggdrasil.listKnownToggles(this.enginePtr),
                new TypeReference<YggResponse<List<FeatureDef>>>() {
                });
        return response.getValue();
    }

    /** Handle reading from a pointer into a String and mapping it to an object */
    private <T> T read(Pointer pointer, TypeReference<T> clazz) {
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
