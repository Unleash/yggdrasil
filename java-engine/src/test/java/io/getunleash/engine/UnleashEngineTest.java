package io.getunleash.engine;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.io.File;
import java.io.IOException;
import java.nio.file.Paths;
import java.util.List;
import java.util.Map;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import io.getunleash.engine.UnleashEngine;
import io.getunleash.engine.VariantDef;

class UnleashEngineTest {

    private String simpleFeatures = loadFeaturesFromFile(
            "../../client-specification/specifications/01-simple-examples.json"); // Assume this is set up to be your
                                                                                     // feature JSON

    public static String loadFeaturesFromFile(String filePath) {
        ObjectMapper mapper = new ObjectMapper();
        try {
            JsonNode jsonNode = mapper.readTree(Paths.get(filePath).toFile());
            JsonNode state = jsonNode.get("state");
            return state.toString();
        } catch (IOException e) {
            e.printStackTrace();
            return null;
        }
    }

    @Test
    void testTakeState() {
        UnleashEngine engine = new UnleashEngine();
        engine.takeState(simpleFeatures);
    }

    @Test
    void testIsEnabled() throws Exception {
        UnleashEngine engine = new UnleashEngine();
        engine.takeState(simpleFeatures);

        Context context = new Context();
        boolean result = engine.isEnabled("Feature.A", context);
        assertTrue(result);
    }

    @Test
    void testGetVariant() throws Exception {
        UnleashEngine engine = new UnleashEngine();
        engine.takeState(simpleFeatures);

        Context context = new Context();
        VariantDef variant = engine.getVariant("Feature.A", context);

        assertEquals("disabled", variant.name);
        assertFalse(variant.enabled);
    }

    @Test
    public void testClientSpec() throws Exception {
        UnleashEngine unleashEngine = new UnleashEngine();
        ObjectMapper objectMapper = new ObjectMapper();
        File basePath = Paths.get( "..", "..", "client-specification", "specifications").toFile();
        File indexFile = new File(basePath, "index.json");
        List<String> testSuites = objectMapper.readValue(indexFile, new TypeReference<>() {
        });

        for (String suite : testSuites) {
            File suiteFile = new File(basePath, suite);
            Map<String, Object> suiteData = objectMapper.readValue(suiteFile, new TypeReference<>() {
            });

            unleashEngine.takeState(objectMapper.writeValueAsString(suiteData.get("state")));

            List<Map<String, Object>> tests = (List<Map<String, Object>>) suiteData.get("tests");
            if (tests != null) {
                for (Map<String, Object> test : tests) {
                    String contextJson = objectMapper.writeValueAsString(test.get("context"));
                    Context context = objectMapper.readValue(contextJson, Context.class);
                    String toggleName = (String) test.get("toggleName");
                    boolean expectedResult = (Boolean) test.get("expectedResult");

                    boolean result = unleashEngine.isEnabled(toggleName, context);

                    assertEquals(expectedResult, result,
                            String.format("[%s] Failed test '%s': expected %b, got %b",
                                    suite,
                                    test.get("description"), expectedResult,
                                    result));
                }
            }

            List<Map<String, Object>> variantTests = (List<Map<String, Object>>) suiteData.get("variantTests");
            if (variantTests != null) {
                for (Map<String, Object> test : variantTests) {
                    String contextJson = objectMapper.writeValueAsString(test.get("context"));
                    Context context = objectMapper.readValue(contextJson, Context.class);
                    String toggleName = (String) test.get("toggleName");

                    VariantDef expectedResult = objectMapper.convertValue(test.get("expectedResult"), VariantDef.class);
                    VariantDef result = unleashEngine.getVariant(toggleName, context);

                    String expectedResultJson = objectMapper.writeValueAsString(expectedResult);
                    String resultJson = objectMapper.writeValueAsString(result);

                    assertEquals(expectedResultJson, resultJson,
                            String.format("Failed test '%s': expected %b, got %b",
                                    test.get("description"), expectedResultJson,
                                    resultJson));
                }
            }

            System.out.println(String.format("Completed specification '%s'", suite));
        }
    }
}
