package io.getunleash.engine;

public class YggdrasilParseException extends RuntimeException {
    public <T> YggdrasilParseException(String input, Class<T> target, Exception parent) {
        super("Can't read " + input + " into " + target, parent);
    }
}
