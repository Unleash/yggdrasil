package io.getunleash.engine;

public class YggdrasilInvalidInputException extends Exception {
    public YggdrasilInvalidInputException(String input) {
        super("The input provided is invalid: "+input);
    }

    public YggdrasilInvalidInputException(Context input) {
        super("The context provided is invalid: "+input);
    }
}
