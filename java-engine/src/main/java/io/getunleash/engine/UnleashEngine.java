package io.getunleash.engine;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import com.sun.jna.Pointer;
import java.io.IOException;
import java.util.*;
import java.util.stream.Stream;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class UnleashEngine {
    private static final String UTF_8 = "UTF-8";
    private static final String EMPTY_STRATEGY_RESULTS = "{}";

    private static final Logger log = LoggerFactory.getLogger(UnleashEngine.class);
    private final YggdrasilFFI yggdrasil;

    private final ObjectMapper mapper;
    private final CustomStrategies customStrategies;

    public UnleashEngine() {
        this(new YggdrasilFFI());
    }

    public UnleashEngine(List<IStrategy> customStrategies) {
        this(new YggdrasilFFI(), customStrategies);
    }

    UnleashEngine(YggdrasilFFI yggdrasil) {
        this(yggdrasil, null);
    }

    UnleashEngine(YggdrasilFFI yggdrasil, List<IStrategy> customStrategies) {
        this.yggdrasil = yggdrasil;
        this.mapper = new ObjectMapper();
        this.mapper.registerModule(new JavaTimeModule());
        this.mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
        if (customStrategies != null && !customStrategies.isEmpty()) {
            List<String> builtInStrategies =
                    read(yggdrasil.builtInStrategies(), new TypeReference<>() {});
            this.customStrategies =
                    new CustomStrategies(
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
                                            }));
        } else {
            this.customStrategies = new CustomStrategies(Stream.empty());
        }
    }

    public void takeState(String toggles) throws YggdrasilInvalidInputException {
        YggResponse<Void> response = read(yggdrasil.takeState(toggles), new TypeReference<>() {});
        if (!response.isValid()) {
            throw new YggdrasilInvalidInputException(toggles);
        }

        customStrategies.takeState(toggles);
    }

    public Boolean isEnabled(String name, Context context)
            throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            String strategyResults = customStrategies.eval(name, context);
            YggResponse<Boolean> isEnabled =
                    read(
                            yggdrasil.checkEnabled(name, jsonContext, strategyResults),
                            new TypeReference<>() {});
            return isEnabled.getValue();
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public VariantDef getVariant(String name, Context context)
            throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            YggResponse<VariantDef> response =
                    read(
                            yggdrasil.checkVariant(name, jsonContext, EMPTY_STRATEGY_RESULTS),
                            new TypeReference<>() {});
            return response.getValue();
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public void countToggle(String flagName, boolean enabled) {
        this.yggdrasil.countToggle(flagName, enabled);
    }

    public void countVariant(String flagName, String variantName) {
        this.yggdrasil.countVariant(flagName, variantName);
    }

    public MetricsBucket getMetrics() throws YggdrasilError {
        YggResponse<MetricsBucket> response =
                read(yggdrasil.getMetrics(), new TypeReference<>() {});
        return response.getValue();
    }

    public boolean shouldEmitImpressionEvent(String name) throws YggdrasilError {
        YggResponse<Boolean> response =
                read(yggdrasil.shouldEmitImpressionEvent(name), new TypeReference<>() {});
        return response.getValue();
    }

    /** Handle reading from a pointer into a String and mapping it to an object */
    private <T> T read(Pointer pointer, TypeReference<T> clazz) {
        try {
            String str = pointer.getString(0, UTF_8);
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
}
