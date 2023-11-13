package io.getunleash.engine;


import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.time.Instant;
import java.util.Map;

public class MetricsBucket {
    private final Instant start;
    private final Instant stop;
    private final Map<String, FeatureCount> toggles;

    @JsonCreator
    public MetricsBucket(
            @JsonProperty("start") Instant start,
            @JsonProperty("stop") Instant stop,
            @JsonProperty("toggles") Map<String, FeatureCount> toggles) {
        this.start = start;
        this.stop = stop;
        this.toggles = toggles;
    }

    public Instant getStart() {
        return start;
    }

    public Instant getStop() {
        return stop;
    }

    public Map<String, FeatureCount> getToggles() {
        return toggles;
    }
}
