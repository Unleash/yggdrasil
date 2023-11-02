package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

public class VariantResponse extends YggResponse<VariantDef> {

    @JsonCreator
    VariantResponse(
            @JsonProperty("status_code") StatusCode statusCode,
            @JsonProperty("value") VariantDef value,
            @JsonProperty("error_message") String errorMessage
    ) {
        super(statusCode, value, errorMessage);
    }

    public String getName() {
        return this.value.getName();
    }

    public Payload getPayload() {
        if (isValid()) {
            return this.value.getPayload();
        } else {
            return null;
        }
    }

    public Boolean isEnabled() {
        if (isValid()) {
            return this.value.getEnabled();
        } else {
            return false;
        }
    }
}