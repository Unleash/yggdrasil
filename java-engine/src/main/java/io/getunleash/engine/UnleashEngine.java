package io.getunleash.engine;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.jna.Pointer;

public class UnleashEngine {
    private static final String UTF_8 = "UTF-8";
    private final YggdrasilFFI yggdrasil;
    private final ObjectMapper mapper;

    public UnleashEngine() {
        this(new YggdrasilFFI());
    }

    UnleashEngine(YggdrasilFFI yggdrasil) {
        this.yggdrasil = yggdrasil;
        mapper = new ObjectMapper();
        mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
    }

    public void takeState(String toggles) throws YggdrasilInvalidInputException {
        TakeStateResponse response = read(yggdrasil.takeState(toggles), TakeStateResponse.class);
        if (!response.isValid()) {
            throw new YggdrasilInvalidInputException(toggles);
        }
    }

    public boolean isEnabled(String name, Context context) throws YggdrasilInvalidInputException {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            IsEnabledResponse isEnabled = read(yggdrasil.checkEnabled(name, jsonContext), IsEnabledResponse.class);
            return isEnabled.isEnabled();
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    public VariantResponse getVariant(String name, Context context) throws YggdrasilInvalidInputException {
        try {
            String jsonContext = mapper.writeValueAsString(context);
            return read(yggdrasil.checkVariant(name, jsonContext), VariantResponse.class);
        } catch (JsonProcessingException e) {
            throw new YggdrasilInvalidInputException(context);
        }
    }

    /**
     * Handle reading from a pointer into a String and mapping it to an object
     */
    private <T> T read(Pointer pointer, Class<T> clazz) {
        String str = pointer.getString(0, UTF_8);
        yggdrasil.freeResponse(pointer);
        try {
            System.out.println(str); // TODO use a logging library. SLF4J?
            return mapper.readValue(str, clazz);
        } catch (JsonProcessingException e) {
            throw new YggdrasilParseException(str, clazz, e);
        }
    }
}