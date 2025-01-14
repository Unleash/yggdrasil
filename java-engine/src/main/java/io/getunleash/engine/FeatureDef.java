package io.getunleash.engine;

import java.util.Optional;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

class FeatureDef {
    private final String name;
    private final Optional<String> type;
    private final String project;

    @JsonCreator
    FeatureDef(
            @JsonProperty("name") String name,
            @JsonProperty("type") String featureType,
            @JsonProperty("project") String project) {
        this.name = name;
        this.project = project;
        this.type = Optional.ofNullable(featureType);
    }

    public String getName() {
        return name;
    }

    public Optional<String> getType() {
        return type;
    }

    public String getProject() {
        return project;
    }
}