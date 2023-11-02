package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

public class MetricsResponse extends YggResponse<MetricsBucket> {

    @JsonCreator
    MetricsResponse(@JsonProperty("status_code") StatusCode statusCode, @JsonProperty("value") MetricsBucket value, @JsonProperty("error_message") String errorMessage) {
        super(statusCode, value, errorMessage);
    }

    public MetricsBucket getValue() {
        if (isValid()) {
            return this.value;
        } else {
            return null;
        }
    }
}
