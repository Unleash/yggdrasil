package io.getunleash.engine;

public enum StatusCode {
    OK("Ok"),
    NOT_FOUND("NotFound"),
    ERROR("Error");

    private final String name;

    StatusCode(String name) {
        this.name = name;
    }

    public static StatusCode fromName(String name) {
        for (StatusCode status : StatusCode.values()) {
            if (status.name.equalsIgnoreCase(name)) {
                return status;
            }
        }
        throw new IllegalArgumentException("Unknown status code: " + name);
    }
}