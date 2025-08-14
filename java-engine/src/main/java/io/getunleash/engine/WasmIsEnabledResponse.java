package io.getunleash.engine;

public class WasmIsEnabledResponse extends WasmResponse<Boolean> {
  WasmIsEnabledResponse(boolean impressionData, Boolean value) {
    super(impressionData, value);
  }
}
