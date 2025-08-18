package io.getunleash.engine;

public class YggdrasilError extends Exception {
  public YggdrasilError(String message) {
    super(message);
  }

  public YggdrasilError(String message, Throwable cause) {
    super(message, cause);
  }
}
