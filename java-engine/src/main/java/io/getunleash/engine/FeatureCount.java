package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.Map;

public class FeatureCount {
    private final Long yes;
    private final Long no;
    private final Map<String, Long> variants;

    @JsonCreator
    public FeatureCount(
            @JsonProperty("yes") Long yes,
            @JsonProperty("no") Long no,
            @JsonProperty("variants") Map<String, Long> variants) {
        this.yes = yes;
        this.no = no;
        this.variants = variants;
    }

    public Long getYes() {
        return yes;
    }

    public Long getNo() {
        return no;
    }

    public Map<String, Long> getVariants() {
        return variants;
    }
}
