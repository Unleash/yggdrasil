package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonProperty;

public class WasmResponse<T> {
    @JsonProperty("status_code")
    public String statusCode;

    public T value;

    @JsonProperty("error_message")
    public String errorMessage;
}