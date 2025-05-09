package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonProperty;

public class WasmResponse<T> {
  @JsonProperty("status_code")
  public boolean impressionData;

  public T value;

  public WasmResponse(boolean impressionData, T value) {
    this.impressionData = impressionData;
    this.value = value;
  }
}
