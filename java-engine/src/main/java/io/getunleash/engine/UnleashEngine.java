package io.getunleash.engine;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
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
    private final ObjectWriter writer;

    public UnleashEngine() {
        this(new YggdrasilFFI());
    }

    UnleashEngine(YggdrasilFFI yggdrasil) {
        this.yggdrasil = yggdrasil;
        ObjectMapper mapper = new ObjectMapper();
        mapper.registerModule(new JavaTimeModule());
        writer = mapper.writer();
    }

    private ObjectReader getReader() {
        ObjectMapper mapper = new ObjectMapper();
        mapper.registerModule(new JavaTimeModule());
        mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
        return mapper.reader();
    }

    public void takeState(String toggles) throws YggdrasilInvalidInputException {
        YggResponse<Void> response = read(yggdrasil.takeState(toggles), getReader().forType(new TypeReference<YggResponse<Void>>() {}));
        if (!response.isValid()) {
            throw new YggdrasilInvalidInputException(toggles);
        }
    }

    public Boolean isEnabled(String name, Context context) throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = writer.writeValueAsString(context);
            YggResponse<Boolean> isEnabled = read(yggdrasil.checkEnabled(name, jsonContext, "{}"), getReader().forType(new TypeReference<YggResponse<Boolean>>() {}));
            return isEnabled.getValue();
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public VariantDef getVariant(String name, Context context) throws YggdrasilInvalidInputException, YggdrasilError {
        try {
            String jsonContext = writer.writeValueAsString(context);
            YggResponse<VariantDef> response = read(yggdrasil.checkVariant(name, jsonContext, "{}"), getReader().forType(new TypeReference<YggResponse<VariantDef>>() {}));
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
        YggResponse<MetricsBucket> response = read(yggdrasil.getMetrics(), getReader().forType(new TypeReference<YggResponse<MetricsBucket>>() {}));
        return response.getValue();
    }

    /**
     * Handle reading from a pointer into a String and mapping it to an object
     */
    private <T> T read(Pointer pointer, ObjectReader reader) {
        try {
            String str = pointer.getString(0, UTF_8);
            try {
                return reader.readValue(str);
            } catch (IOException e) {
                System.out.println("Failed to parse response from Yggdrasil: " + str);
                throw new YggdrasilParseException(str, reader.getClass(), e);
            }
        } finally {
            yggdrasil.freeResponse(pointer);
        }
    }
}