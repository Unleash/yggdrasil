package io.getunleash.engine;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import com.sun.jna.Pointer;

import java.io.IOException;

public class UnleashEngine {
    private static final String UTF_8 = "UTF-8";
    private static final String CUSTOM_STRATEGY_RESULTS = "{}";
    private final YggdrasilFFI yggdrasil;

    private final ObjectMapper mapper;

    public UnleashEngine() {
        this(new YggdrasilFFI());
    }

    UnleashEngine(YggdrasilFFI yggdrasil) {
        this.yggdrasil = yggdrasil;
        this.mapper = new ObjectMapper();
        mapper.registerModule(new JavaTimeModule());
        mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
    }

    public void takeState(String toggles) throws YggdrasilInvalidInputException {
        YggResponse<Void> response = read(yggdrasil.takeState(toggles), new TypeReference<>() {});
        if (!response.isValid()) {
            throw new YggdrasilInvalidInputException(toggles);
        }
    }

    public Boolean isEnabled(String name, Context context) throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            YggResponse<Boolean> isEnabled = read(yggdrasil.checkEnabled(name, jsonContext, CUSTOM_STRATEGY_RESULTS), new TypeReference<>() {});
            return isEnabled.getValue();
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public VariantDef getVariant(String name, Context context) throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            YggResponse<VariantDef> response = read(yggdrasil.checkVariant(name, jsonContext, CUSTOM_STRATEGY_RESULTS), new TypeReference<>() {});
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
        YggResponse<MetricsBucket> response = read(yggdrasil.getMetrics(), new TypeReference<>() {
        });
        return response.getValue();
    }

    /**
     * Handle reading from a pointer into a String and mapping it to an object
     */
    private <T> T read(Pointer pointer, TypeReference<T> clazz) {
        try {
            String str = pointer.getString(0, UTF_8);
            try {
                return mapper.readValue(str, clazz);
            } catch (IOException e) {
                System.out.println("Failed to parse response from Yggdrasil: " + str);
                throw new YggdrasilParseException(str, clazz.getClass(), e);
            }
        } finally {
            yggdrasil.freeResponse(pointer);
        }
    }
}