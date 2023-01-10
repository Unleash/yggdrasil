package io.getunleash.javasdk;

public class VariantTestCase {
    private String description;
    private Context context;
    private String toggleName;
    private Variant expectedResult;

    public Context getContext() {
        return context;
    }

    public String getDescription() {
        return description;
    }

    public String getToggleName() {
        return toggleName;
    }

    public Variant getExpectedResult() {
        if (expectedResult.getName().equals("disabled")) {
            return Variant.DISABLED_VARIANT;
        }

        return expectedResult;
    }
}
