package io.getunleash.javasdk;

public class TestCase {
    private String description;
    private Context context;
    private String toggleName;
    private boolean expectedResult;

    public Context getContext() {
        return context;
    }

    public String getDescription() {
        return description;
    }

    public String getToggleName() {
        return toggleName;
    }

    public boolean getExpectedResult() {
        return expectedResult;
    }
}