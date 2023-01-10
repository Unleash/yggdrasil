package io.getunleash.javasdk;

import static org.junit.Assert.assertEquals;

import java.io.BufferedReader;
import java.io.FileInputStream;
import java.io.InputStreamReader;

import org.junit.Test;

import com.google.gson.Gson;

public class UnleashEngineTest {

    private static String SPEC_PATH = "../client-specification/specifications/";

    // Seems like a silly test but it's important, Rust will do everything in its
    // power to deallocate memory it owns
    // which means that if the called Rust function holds ownership of the memory
    // after the method call the
    // underlying struct will be deallocated without notice to the Java code,
    // meaning the second isEnabled will crash
    @Test
    public void shouldBeAbleToCallIsEnabledTwice() {
        UnleashEngine engine = new UnleashEngine();
        boolean thing = engine.isEnabled("test", new Context());
        boolean thing2 = engine.isEnabled("test", new Context());
        assertEquals(thing, false);
        assertEquals(thing2, false);
    }

    @Test
    public void acceptsANullContext() {
        UnleashEngine engine = new UnleashEngine();
    }

    @Test
    public void runSpecTests() throws Exception {
        SpecDefinition def = getSpec("01-simple-examples.json");
        UnleashEngine engine = new UnleashEngine();
        engine.takeState(def.getState().toString());
        for (TestCase test : def.getTests()) {
            boolean isEnabled = engine.isEnabled(test.getToggleName(), test.getContext());
            assertEquals(isEnabled, test.getExpectedResult());
        }
    }

    public List<String> getAllSpecs() throws Exception {
        try (FileInputStream inputStream = new FileInputStream("../")) {
            InputStreamReader reader = new InputStreamReader(inputStream);
            
        }
    }

    public SpecDefinition getSpec(String fileName) throws Exception {
        try (FileInputStream inputStream = new FileInputStream(
                SPEC_PATH + fileName)) {
            InputStreamReader reader = new InputStreamReader(inputStream);
            return new Gson().fromJson(new BufferedReader(reader), SpecDefinition.class);
        }
    }
}
