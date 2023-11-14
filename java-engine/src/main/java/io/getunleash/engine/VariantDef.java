package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

public class VariantDef {
    private final String name;
    private final Payload payload;
    private final Boolean enabled;

    @JsonCreator
    VariantDef(
            @JsonProperty("name") String name,
            @JsonProperty("payload") Payload payload,
            @JsonProperty("enabled") Boolean enabled) {
        this.name = name;
        this.payload = payload;
        this.enabled = enabled;
    }

    public String getName() {
        return name;
    }

    public Payload getPayload() {
        return payload;
    }

    public Boolean isEnabled() {
        return enabled;
    }
}
