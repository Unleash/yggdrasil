package io.getunleash.engine;

import static org.junit.jupiter.api.Assertions.*;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.File;
import java.io.IOException;
import java.net.URISyntaxException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.time.Instant;
import java.time.temporal.ChronoUnit;
import java.util.List;
import java.util.Map;
import java.util.Objects;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class TestSuite {
    public String name;
    public Object state;
    public List<Map<String, Object>> tests;
    public List<Map<String, Object>> variantTests;
}

class UnleashEngineTest {

    private static final VariantDef DEFAULT_VARIANT = new VariantDef("disabled", null, false);

    // Assume this is set up to be your feature JSON
    private final String simpleFeatures =
            loadFeaturesFromFile("../client-specification/specifications/01-simple-examples.json");

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

    @Test
    void testTakeState() throws YggdrasilInvalidInputException {
        engine.takeState(simpleFeatures);
    }

    @Test
    void testIsEnabled() throws Exception {
        engine.takeState(simpleFeatures);

        Context context = new Context();
        Boolean result = engine.isEnabled("Feature.A", context);
        assertNotNull(result);
        assertTrue(result);
    }

    @Test
    void testIsEnabledWithoutValidResponse() throws Exception {
        engine.takeState(simpleFeatures);

        Context context = new Context();
        Boolean result = engine.isEnabled("IDoNotExist", context);
        assertNull(result); // not found
    }

    @Test
    void testGetVariant() throws Exception {
        engine.takeState(simpleFeatures);

        Context context = new Context();
        VariantDef variant = engine.getVariant("Feature.A", context);

        if (variant == null) {
            variant = DEFAULT_VARIANT;
        }

        assertEquals("disabled", variant.getName());
        assertFalse(variant.isEnabled());
    }

    @Test
    public void testClientSpec() throws Exception {
        ObjectMapper objectMapper = new ObjectMapper();
        File basePath = Paths.get("../client-specification/specifications").toFile();
        File indexFile = new File(basePath, "index.json");
        List<String> testSuites = objectMapper.readValue(indexFile, new TypeReference<>() {});

        for (String suite : testSuites) {
            File suiteFile = new File(basePath, suite);
            TestSuite suiteData = objectMapper.readValue(suiteFile, new TypeReference<>() {});

            engine.takeState(objectMapper.writeValueAsString(suiteData.state));

            List<Map<String, Object>> tests = suiteData.tests;
            if (tests != null) {
                for (Map<String, Object> test : tests) {
                    String contextJson = objectMapper.writeValueAsString(test.get("context"));
                    Context context = objectMapper.readValue(contextJson, Context.class);
                    String toggleName = (String) test.get("toggleName");
                    boolean expectedResult = (Boolean) test.get("expectedResult");

                    Boolean result = engine.isEnabled(toggleName, context);

                    if (result == null) {
                        result = false; // Default should be provided by SDK
                    }

                    assertEquals(
                            expectedResult,
                            result,
                            String.format(
                                    "[%s] Failed test '%s': expected %b, got %b",
                                    suiteData.name,
                                    test.get("description"),
                                    expectedResult,
                                    result));
                }
            }

            List<Map<String, Object>> variantTests = suiteData.variantTests;
            if (variantTests != null) {
                for (Map<String, Object> test : variantTests) {
                    String contextJson = objectMapper.writeValueAsString(test.get("context"));
                    Context context = objectMapper.readValue(contextJson, Context.class);
                    String toggleName = (String) test.get("toggleName");

                    VariantDef expectedResult =
                            objectMapper.convertValue(test.get("expectedResult"), VariantDef.class);
                    VariantDef result = engine.getVariant(toggleName, context);
                    if (result == null) {
                        // this behavior should be implemented in the SDK
                        result = DEFAULT_VARIANT;
                    }

                    String expectedResultJson = objectMapper.writeValueAsString(expectedResult);
                    String resultJson = objectMapper.writeValueAsString(result);

                    assertEquals(
                            expectedResultJson,
                            resultJson,
                            String.format(
                                    "[%s] Failed test '%s': expected %b, got %b",
                                    suiteData.name,
                                    test.get("description"),
                                    expectedResultJson,
                                    resultJson));
                }
            }

            System.out.printf("Completed specification '%s'%n", suite);
        }
    }

    @Test
    void testMetrics() throws YggdrasilError {
        engine.countVariant("Feature.A", "A");
        engine.countToggle("Feature.B", true);
        engine.countToggle("Feature.C", false);
        engine.countToggle("Feature.C", false);
        MetricsBucket bucket = engine.getMetrics();

        assertNotNull(bucket);

        Instant start = bucket.getStart();
        Instant stop = bucket.getStop();
        assertNotNull(start);
        assertNotNull(stop);
        assertTrue(stop.isAfter(start)); // unlikely to be equal but could happen
        assertTrue(
                start.until(Instant.now(), ChronoUnit.SECONDS)
                        < 10); // should be within 10 seconds of now

        assertEquals(3, bucket.getToggles().size());

        assertEquals(1, bucket.getToggles().get("Feature.A").getVariants().get("A"));
        // Validate: counting on enabled is up to the SDK or should we also count enabled when
        // getting a variant?
        assertEquals(0, bucket.getToggles().get("Feature.A").getYes());
        assertEquals(0, bucket.getToggles().get("Feature.A").getNo());

        assertEquals(1, bucket.getToggles().get("Feature.B").getYes());
        assertEquals(0, bucket.getToggles().get("Feature.B").getNo());

        assertEquals(0, bucket.getToggles().get("Feature.C").getYes());
        assertEquals(2, bucket.getToggles().get("Feature.C").getNo());
    }

    @Test
    void testImpressionData() throws Exception {
        String features = Files.readString(Paths.get(Objects.requireNonNull(getClass().getClassLoader().getResource("impression-data-tests.json")).toURI()));
        String featureName = "with.impression.data";

        assertFalse(engine.shouldEmitImpressionEvent(featureName));

        engine.takeState(features);
        Boolean result = engine.isEnabled(featureName, new Context());
        assertNotNull(result);
        assertTrue(result);
        assertTrue(engine.shouldEmitImpressionEvent(featureName));
    }

    @Test
    void testImpressionDataFalse() throws Exception {
        String features = Files.readString(Paths.get(Objects.requireNonNull(getClass().getClassLoader().getResource("impression-data-tests.json")).toURI()));
        String featureName = "with.impression.data.false";

        assertFalse(engine.shouldEmitImpressionEvent(featureName));

        engine.takeState(features);
        Boolean result = engine.isEnabled(featureName, new Context());
        assertNotNull(result);
        assertTrue(result);
        assertFalse(engine.shouldEmitImpressionEvent(featureName));
    }
}
