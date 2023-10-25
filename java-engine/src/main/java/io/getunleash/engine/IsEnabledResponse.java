package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

class IsEnabledResponse extends YggResponse<Boolean> {

    @JsonCreator(mode = JsonCreator.Mode.PROPERTIES)
    IsEnabledResponse(@JsonProperty("status_code") StatusCode statusCode, @JsonProperty("value")  Boolean value, @JsonProperty("error_message") String errorMessage) {
        super(statusCode, value, errorMessage);
    }

    boolean isEnabled() {
        return StatusCode.Ok.equals(this.statusCode) && Boolean.TRUE.equals(this.value);
    }
}
