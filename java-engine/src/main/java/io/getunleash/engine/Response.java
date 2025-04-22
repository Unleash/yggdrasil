package io.getunleash.engine;

public class Response<T> {
    private final T value;
    private final int statusCode;
    private final String errorMessage;

    public Response(T value, int statusCode, String errorMessage) {
        this.value = value;
        this.statusCode = statusCode;
        this.errorMessage = errorMessage;
    }

    public T getValue() {
        return value;
    }

    public int getStatusCode() {
        return statusCode;
    }

    public String getErrorMessage() {
        return errorMessage;
    }
}
