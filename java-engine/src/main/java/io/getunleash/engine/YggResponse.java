package io.getunleash.engine;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

class YggResponse<T> {
  final StatusCode statusCode;
  final T value;
  final String errorMessage;

  @JsonCreator
  YggResponse(
      @JsonProperty("status_code") StatusCode statusCode,
      @JsonProperty("value") T value,
      @JsonProperty("error_message") String errorMessage) {
    this.statusCode = statusCode;
    this.value = value;
    this.errorMessage = errorMessage;
  }

  boolean isValid() {
    return !StatusCode.Error.equals(this.statusCode);
  }

  public T getValue() throws YggdrasilError {
    if (isValid()) {
      return this.value;
    } else {
      if (this.statusCode == null || this.statusCode.equals(StatusCode.Error)) {
        throw new IllegalStateException("statusCode is null");
      }
      return null;
    }
  }

  @Override
  public String toString() {
    return "YggResponse{"
        + "statusCode="
        + statusCode
        + ", value="
        + value
        + ", errorMessage='"
        + errorMessage
        + '\''
        + '}';
  }
}
