package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.Optional;

public class VariantDef {
    private final String name;
    private final Payload payload;
    private final Boolean enabled;
    private final Boolean featureEnabled;

    @JsonCreator
    VariantDef(
            @JsonProperty("name") String name,
            @JsonProperty("payload") Payload payload,
            @JsonProperty("enabled") Boolean enabled,
            @JsonProperty("feature_enabled") Boolean featureEnabled) {
        this.name = name;
        this.payload = payload;
        this.enabled = enabled;
        this.featureEnabled = Optional.ofNullable(featureEnabled).orElse(false);
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

    public Boolean isFeatureEnabled() {
        return featureEnabled;
    }
}
