package io.getunleash.engine;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.ObjectReader;
import com.fasterxml.jackson.databind.ObjectWriter;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import com.sun.jna.Pointer;

import java.io.IOException;

public class UnleashEngine {
    private static final String UTF_8 = "UTF-8";
    private final YggdrasilFFI yggdrasil;
    private final ObjectReader reader;
    private final ObjectWriter writer;

    public UnleashEngine() {
        this(new YggdrasilFFI());
    }

    UnleashEngine(YggdrasilFFI yggdrasil) {
        this.yggdrasil = yggdrasil;
        ObjectMapper mapper = new ObjectMapper();
        mapper.registerModule(new JavaTimeModule());
        mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
        reader = mapper.reader();
        writer = mapper.writer();
    }

    public void takeState(String toggles) throws YggdrasilInvalidInputException {
        TakeStateResponse response = read(yggdrasil.takeState(toggles), TakeStateResponse.class);
        if (!response.isValid()) {
            throw new YggdrasilInvalidInputException(toggles);
        }
    }

    public IsEnabledResponse isEnabled(String name, Context context) throws YggdrasilInvalidInputException {
        try {
            String jsonContext = writer.writeValueAsString(context);
            return read(yggdrasil.checkEnabled(name, jsonContext), IsEnabledResponse.class);
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public VariantResponse getVariant(String name, Context context) throws YggdrasilInvalidInputException {
        try {
            String jsonContext = writer.writeValueAsString(context);
            return read(yggdrasil.checkVariant(name, jsonContext), VariantResponse.class);
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

    public MetricsResponse getMetrics() {
        return read(yggdrasil.getMetrics(), MetricsResponse.class);
    }

    /**
     * Handle reading from a pointer into a String and mapping it to an object
     */
    private <T> T read(Pointer pointer, Class<T> clazz) {
        String str = pointer.getString(0, UTF_8);
        yggdrasil.freeResponse(pointer);
        try {
            System.out.println(str); // TODO use a logging library. SLF4J?
            return reader.readValue(str, clazz);
        } catch (IOException e) {
            throw new YggdrasilParseException(str, clazz, e);
        }
    }
}