package io.getunleash.javasdk;

import java.util.ArrayList;
import java.util.List;

import com.google.gson.JsonObject;

public class SpecDefinition {
    private String name;
    private JsonObject state;
    private List<TestCase> tests = new ArrayList<>();
    private List<VariantTestCase> variantTests = new ArrayList<>();

    public String getName() {
        return name;
    }

    public JsonObject getState() {
        return state;
    }

    public List<TestCase> getTests() {
        return tests;
    }

    public List<VariantTestCase> getVariantTests() {
        return variantTests;
    }
}

