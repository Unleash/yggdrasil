package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

class TakeStateResponse extends YggResponse<Boolean> {
    @JsonCreator(mode = JsonCreator.Mode.PROPERTIES)
    TakeStateResponse(@JsonProperty("status_code") StatusCode statusCode, @JsonProperty("value")  Boolean value, @JsonProperty("error_message") String errorMessage) {
        super(statusCode, value, errorMessage);
    }
}
