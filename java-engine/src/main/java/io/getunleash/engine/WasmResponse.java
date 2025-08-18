package io.getunleash.engine;

class WasmResponse<T> {
  public boolean impressionData;

  public T value;

  WasmResponse(boolean impressionData, T value) {
    this.impressionData = impressionData;
    this.value = value;
  }
}
