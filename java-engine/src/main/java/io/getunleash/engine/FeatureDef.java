package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.Optional;

public class FeatureDef {
  private final String name;
  private final Optional<String> type;
  private final String project;
  private final boolean enabled;

  @JsonCreator
  FeatureDef(
      @JsonProperty("name") String name,
      @JsonProperty("type") String featureType,
      @JsonProperty("project") String project,
      @JsonProperty("enabled") boolean enabled) {
    this.name = name;
    this.project = project;
    this.type = Optional.ofNullable(featureType);
    this.enabled = enabled;
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

  public boolean isEnabled() {
    return enabled;
  }
}
