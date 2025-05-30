package io.getunleash.engine;

import static io.getunleash.engine.TestStrategies.*;
import static org.junit.jupiter.api.Assertions.*;
import static org.junit.jupiter.params.provider.Arguments.of;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.File;
import java.io.IOException;
import java.lang.ref.PhantomReference;
import java.lang.ref.Reference;
import java.lang.ref.ReferenceQueue;
import java.net.URISyntaxException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.time.Instant;
import java.time.temporal.ChronoUnit;
import java.util.*;
import java.util.concurrent.CountDownLatch;
import java.util.stream.Stream;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.CsvSource;
import org.junit.jupiter.params.provider.MethodSource;
import org.mockito.Mockito;

class TestSuite {
  public String name;
  public Object state;
  public List<Map<String, Object>> tests;
  public List<Map<String, Object>> variantTests;
}

class UnleashEngineTest {

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
    List<IStrategy> customStrategies = List.of(alwaysTrue("custom"));
    engine = new UnleashEngine(customStrategies);
  }

  @Test
  void testTakeState() throws YggdrasilInvalidInputException, YggdrasilError {
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
      variant = new VariantDef("disabled", null, false, engine.isEnabled("Feature.A", context));
    }

    assertEquals("disabled", variant.getName());
    assertFalse(variant.isEnabled());
  }

  @Test
  void testGetVariantWithCustomStrategy() throws Exception {
    engine.takeState(
        "{\"version\":1,\"features\":[{\"name\":\"Feature.D\",\"description\":\"Has a custom strategy\",\"enabled\":true,\"strategies\":[{\"name\":\"custom\",\"constraints\":[],\"parameters\":{\"foo\":\"bar\"}}]}]}");

    Context context = new Context();
    VariantDef variant = engine.getVariant("Feature.D", context);

    assertEquals(variant.isFeatureEnabled(), true);
    assertFalse(variant.isEnabled());
  }

  @Test
  void testListKnownTogglesReturnsAllFeatures() throws Exception {
    engine.takeState(
        "{\"version\":1,\"features\":[{\"name\":\"Feature.A\",\"type\":\"experiment\",\"description\":\"Enabled toggle\",\"project\":\"test\",\"enabled\":true,\"strategies\":[{\"name\":\"default\"}]}]}");
    List<FeatureDef> features = engine.listKnownToggles();
    assertEquals(1, features.size());

    Optional<FeatureDef> featureA =
        features.stream().filter(f -> f.getName().equals("Feature.A")).findFirst();
    assertTrue(featureA.isPresent());
    assertEquals("Feature.A", featureA.get().getName());
    assertEquals("test", featureA.get().getProject());
    assertTrue(featureA.get().getType().isPresent());
    assertTrue(featureA.get().isEnabled());
    assertEquals("experiment", featureA.get().getType().get());
  }

  @Test
  public void testClientSpec() throws Exception {
    ObjectMapper objectMapper = new ObjectMapper();
    objectMapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
    File basePath = Paths.get("../client-specification/specifications").toFile();
    File indexFile = new File(basePath, "index.json");
    List<String> testSuites =
        objectMapper.readValue(indexFile, new TypeReference<List<String>>() {});

    for (String suite : testSuites) {
      File suiteFile = new File(basePath, suite);
      TestSuite suiteData = objectMapper.readValue(suiteFile, new TypeReference<TestSuite>() {});

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
                  suiteData.name, test.get("description"), expectedResult, result));
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
            result = new VariantDef("disabled", null, false, engine.isEnabled(toggleName, context));
          }

          String expectedResultJson = objectMapper.writeValueAsString(expectedResult);
          String resultJson = objectMapper.writeValueAsString(result);

          assertEquals(
              expectedResultJson,
              resultJson,
              String.format(
                  "[%s] Failed test '%s': expected %b, got %b",
                  suiteData.name, test.get("description"), expectedResultJson, resultJson));
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
        start.until(Instant.now(), ChronoUnit.SECONDS) < 10); // should be within 10 seconds of now

    assertEquals(3, bucket.getToggles().size());

    assertEquals(1, bucket.getToggles().get("Feature.A").getVariants().get("A"));
    // Validate: counting on enabled is up to the SDK or should we also count
    // enabled when
    // getting a variant?
    assertEquals(0, bucket.getToggles().get("Feature.A").getYes());
    assertEquals(0, bucket.getToggles().get("Feature.A").getNo());

    assertEquals(1, bucket.getToggles().get("Feature.B").getYes());
    assertEquals(0, bucket.getToggles().get("Feature.B").getNo());

    assertEquals(0, bucket.getToggles().get("Feature.C").getYes());
    assertEquals(2, bucket.getToggles().get("Feature.C").getNo());
  }

  @ParameterizedTest
  @CsvSource({
    "with.impression.data, true",
    "with.impression.data.false, false",
    "with.impression.data.undefined, false"
  })
  void impressionData_whenFeature_shouldReturn(String featureName, boolean expectedImpressionData)
      throws Exception {
    assertFalse(engine.shouldEmitImpressionEvent(featureName));

    takeFeaturesFromResource(engine, "impression-data-tests.json");
    Boolean result = engine.isEnabled(featureName, new Context());
    assertNotNull(result);
    assertTrue(result);
    assertEquals(expectedImpressionData, engine.shouldEmitImpressionEvent(featureName));
  }

  @ParameterizedTest
  @MethodSource("customStrategiesInput")
  void customStrategiesRequired_whenNotConfigured_returnsFalse(
      List<IStrategy> customStrategies,
      String featureName,
      Context context,
      boolean expectedIsEnabled)
      throws Exception {
    UnleashEngine customEngine = new UnleashEngine(customStrategies);
    takeFeaturesFromResource(customEngine, "custom-strategy-tests.json");
    Boolean result = customEngine.isEnabled(featureName, context);
    assertNotNull(result);
    assertEquals(expectedIsEnabled, result);
  }

  @SuppressWarnings("unused")
  @Test
  void testResourceCleanup() throws InterruptedException {
    UnleashFFI ffiMock = Mockito.mock(UnleashFFI.class);
    ReferenceQueue<UnleashEngine> queue = new ReferenceQueue<>();

    UnleashEngine library = new UnleashEngine(ffiMock, null, null);
    PhantomReference<UnleashEngine> reference = new PhantomReference<>(library, queue);

    // Make the object eligible for garbage collection
    library = null;
    Reference<? extends UnleashEngine> polledReference = null;

    for (int i = 0; i < 50; i++) {
      System.gc();
      polledReference = queue.poll();
      if (polledReference != null) {
        break;
      }
      Thread.sleep(10);
    }

    assertNotNull(polledReference, "Cleaner did not trigger");
    Mockito.verify(ffiMock).freeEngine(Mockito.any());
  }

  @Test
  void testCoreVersionIsRetrieved() {
    String coreVersion = UnleashEngine.getCoreVersion();
    assertNotNull(coreVersion);
    // check that it contains two dots, close enough for a quick and dirty but
    // stable semver check
    assertTrue(coreVersion.split("\\.").length >= 3);
  }

  @Test
  void testThreadCollision() throws Exception {
    // This surfaces an issue where calling takeState on the engine in a tight loop
    // from multiple threads causes
    // memory issues like double frees or segfaults
    // that's fixed now but it'd be cool if it didn't come back

    String features = readResource("impression-data-tests.json");
    UnleashEngine ygg = new UnleashEngine();
    int threadCount = 2;
    CountDownLatch latch = new CountDownLatch(threadCount);

    try {
      for (int i = 0; i < 2; i++) {
        new Thread(
                () -> {
                  try {
                    for (int j = 0; j < 100; j++) {
                      ygg.takeState(features);
                    }
                    System.out.println("Thread completed successfully.");
                  } catch (Exception yex) {
                    yex.printStackTrace();
                  } finally {
                    latch.countDown();
                  }
                })
            .start();
      }

      System.out.println("All threads started.");
    } catch (Exception ex) {
      ex.printStackTrace();
    }
    latch.await();
  }

  private static Stream<Arguments> customStrategiesInput() {
    Context oneYesContext = new Context();
    oneYesContext.setProperties(mapOf("one", "yes"));
    return Stream.of(
        of(null, "Feature.Custom.Strategies", new Context(), false),
        of(Collections.emptyList(), "Feature.Custom.Strategies", new Context(), false),
        of(
            Collections.singletonList(alwaysTrue("custom")),
            "Feature.Custom.Strategies",
            new Context(),
            true),
        of(
            Collections.singletonList(onlyTrueIfAllParametersInContext("custom")),
            "Feature.Custom.Strategies",
            new Context(),
            false),
        of(
            Collections.singletonList(onlyTrueIfAllParametersInContext("custom")),
            "Feature.Custom.Strategies",
            oneYesContext,
            true),
        of(
            Collections.singletonList(onlyTrueIfAllParametersInContext("custom")),
            "Feature.Mixed.Strategies",
            oneYesContext,
            true),
        of(
            Collections.singletonList(alwaysTrue("custom")),
            "Feature.Mixed.Strategies",
            oneYesContext,
            true),
        of(Collections.emptyList(), "Feature.Mixed.Strategies", oneYesContext, true),
        of(
            Collections.singletonList(alwaysFails("custom")),
            "Feature.Mixed.Strategies",
            oneYesContext,
            true));
  }

  static Map<String, String> mapOf(String key, String value) {
    return new HashMap<String, String>() {
      {
        put(key, value);
      }
    };
  }

  private void takeFeaturesFromResource(UnleashEngine engine, String resource) {
    try {
      String features = readResource(resource);
      engine.takeState(features);
    } catch (Exception e) {
      throw new RuntimeException("Something went wrong here", e);
    }
  }

  public static String readResource(String resource) throws IOException, URISyntaxException {
    return new String(
        Files.readAllBytes(
            Paths.get(
                Objects.requireNonNull(
                        UnleashEngineTest.class.getClassLoader().getResource(resource))
                    .toURI())),
        StandardCharsets.UTF_8);
  }
}
