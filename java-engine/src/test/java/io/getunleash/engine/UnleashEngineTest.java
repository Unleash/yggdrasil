package io.getunleash.engine;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.io.File;
import java.io.IOException;
import java.nio.file.Paths;
import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;

class TestSuite {
    public String name;
    public Object state;
    public List<Map<String, Object>> tests;
    public List<Map<String, Object>> variantTests;
}

class UnleashEngineTest {

    private static final VariantResponse DEFAULT_VARIANT = new VariantResponse(StatusCode.NotFound, new VariantDef("disabled", null, false), null);
    private final String simpleFeatures = loadFeaturesFromFile(
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

    private UnleashEngine engine;

    @BeforeEach
    void createEngine() {
        engine = new UnleashEngine(new YggdrasilFFI("../target/release"));
    }

    @AfterEach
    void destroy() {
        engine.free();
    }


    @Test
    void testTakeState() throws YggdrasilInvalidInputException {
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
        VariantResponse variant = engine.getVariant("Feature.A", context);

        if (!variant.isValid()) {
            variant = DEFAULT_VARIANT;
        }

        assertEquals("disabled", variant.getName());
        assertFalse(variant.isEnabled());
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
            TestSuite suiteData = objectMapper.readValue(suiteFile, new TypeReference<>() {
            });

            unleashEngine.takeState(objectMapper.writeValueAsString(suiteData.state));

            List<Map<String, Object>> tests = suiteData.tests;
            if (tests != null) {
                for (Map<String, Object> test : tests) {
                    String contextJson = objectMapper.writeValueAsString(test.get("context"));
                    Context context = objectMapper.readValue(contextJson, Context.class);
                    String toggleName = (String) test.get("toggleName");
                    boolean expectedResult = (Boolean) test.get("expectedResult");

                    boolean result = unleashEngine.isEnabled(toggleName, context);

                    assertEquals(expectedResult, result,
                            String.format("[%s] Failed test '%s': expected %b, got %b",
                                    suiteData.name,
                                    test.get("description"), expectedResult,
                                    result));
                }
            }

            List<Map<String, Object>> variantTests = suiteData.variantTests;
            if (variantTests != null) {
                for (Map<String, Object> test : variantTests) {
                    String contextJson = objectMapper.writeValueAsString(test.get("context"));
                    Context context = objectMapper.readValue(contextJson, Context.class);
                    String toggleName = (String) test.get("toggleName");

                    VariantDef expectedResult = objectMapper.convertValue(test.get("expectedResult"), VariantDef.class);
                    VariantResponse result = unleashEngine.getVariant(toggleName, context);
                    if (!result.isValid()) {
                        // this behavior should be implemented in the SDK
                        result = DEFAULT_VARIANT;
                    }

                    String expectedResultJson = objectMapper.writeValueAsString(expectedResult);
                    String resultJson = objectMapper.writeValueAsString(result.value);

                    assertEquals(expectedResultJson, resultJson,
                            String.format("[%s] Failed test '%s': expected %b, got %b",
                                    suiteData.name,
                                    test.get("description"), expectedResultJson,
                                    resultJson));
                }
            }

            System.out.printf("Completed specification '%s'%n", suite);
        }
    }
}
