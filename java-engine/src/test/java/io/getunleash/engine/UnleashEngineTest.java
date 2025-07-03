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
import java.time.ZoneOffset;
import java.time.ZonedDateTime;
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

  String rawState =
      "{\n"
          + //
          "    \"version\": 2,\n"
          + //
          "    \"segments\": [\n"
          + //
          "        {\n"
          + //
          "            \"id\": 1,\n"
          + //
          "            \"name\": \"some-name\",\n"
          + //
          "            \"description\": null,\n"
          + //
          "            \"constraints\": [\n"
          + //
          "                {\n"
          + //
          "                    \"contextName\": \"some-name\",\n"
          + //
          "                    \"operator\": \"IN\",\n"
          + //
          "                    \"value\": \"name\",\n"
          + //
          "                    \"inverted\": false,\n"
          + //
          "                    \"caseInsensitive\": true\n"
          + //
          "                }\n"
          + //
          "            ]\n"
          + //
          "        }\n"
          + //
          "    ],\n"
          + //
          "    \"features\": [\n"
          + //
          "        {\n"
          + //
          "            \"name\": \"Test.old\",\n"
          + //
          "            \"description\": \"No variants here!\",\n"
          + //
          "            \"enabled\": true,\n"
          + //
          "            \"strategies\": [\n"
          + //
          "                {\n"
          + //
          "                    \"name\": \"default\"\n"
          + //
          "                }\n"
          + //
          "            ],\n"
          + //
          "            \"variants\": null,\n"
          + //
          "            \"createdAt\": \"2019-01-24T10:38:10.370Z\"\n"
          + //
          "        },\n"
          + //
          "        {\n"
          + //
          "            \"name\": \"Test.variants\",\n"
          + //
          "            \"description\": null,\n"
          + //
          "            \"enabled\": true,\n"
          + //
          "            \"strategies\": [\n"
          + //
          "                {\n"
          + //
          "                    \"name\": \"default\",\n"
          + //
          "                    \"segments\": [\n"
          + //
          "                        1\n"
          + //
          "                    ]\n"
          + //
          "                }\n"
          + //
          "            ],\n"
          + //
          "            \"variants\": [\n"
          + //
          "                {\n"
          + //
          "                    \"name\": \"variant1\",\n"
          + //
          "                    \"weight\": 50\n"
          + //
          "                },\n"
          + //
          "                {\n"
          + //
          "                    \"name\": \"variant2\",\n"
          + //
          "                    \"weight\": 50\n"
          + //
          "                }\n"
          + //
          "            ],\n"
          + //
          "            \"createdAt\": \"2019-01-24T10:41:45.236Z\"\n"
          + //
          "        },\n"
          + //
          "        {\n"
          + //
          "            \"name\": \"featureX\",\n"
          + //
          "            \"enabled\": true,\n"
          + //
          "            \"strategies\": [\n"
          + //
          "                {\n"
          + //
          "                    \"name\": \"default\"\n"
          + //
          "                }\n"
          + //
          "            ]\n"
          + //
          "        },\n"
          + //
          "        {\n"
          + //
          "            \"name\": \"featureY\",\n"
          + //
          "            \"enabled\": false,\n"
          + //
          "            \"strategies\": [\n"
          + //
          "                {\n"
          + //
          "                    \"name\": \"baz\",\n"
          + //
          "                    \"parameters\": {\n"
          + //
          "                        \"foo\": \"bar\"\n"
          + //
          "                    }\n"
          + //
          "                }\n"
          + //
          "            ]\n"
          + //
          "        },\n"
          + //
          "        {\n"
          + //
          "            \"name\": \"featureZ\",\n"
          + //
          "            \"enabled\": true,\n"
          + //
          "            \"strategies\": [\n"
          + //
          "                {\n"
          + //
          "                    \"name\": \"default\"\n"
          + //
          "                },\n"
          + //
          "                {\n"
          + //
          "                    \"name\": \"hola\",\n"
          + //
          "                    \"parameters\": {\n"
          + //
          "                        \"name\": \"val\"\n"
          + //
          "                    },\n"
          + //
          "                    \"segments\": [\n"
          + //
          "                        1\n"
          + //
          "                    ]\n"
          + //
          "                }\n"
          + //
          "            ]\n"
          + //
          "        }\n"
          + //
          "    ]\n"
          + //
          "}\n"
          + //
          "";

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
    List<IStrategy> customStrategies = new ArrayList<>();
    customStrategies.add(alwaysTrue("custom"));
    engine = new UnleashEngine(customStrategies);
  }

  @Test
  void testTakeState() throws Exception {
    engine.takeState(simpleFeatures);
  }

  @Test
  void testIsEnabled() throws Exception {
    engine.takeState(simpleFeatures);

    Context context = new Context();
    Boolean result = engine.isEnabled("Feature.A", context).value;
    assertNotNull(result);
    assertTrue(result);
  }

  @Test
  void testIsEnabledWithoutValidResponse() throws Exception {
    engine.takeState(simpleFeatures);

    Context context = new Context();
    Boolean result = engine.isEnabled("IDoNotExist", context).value;
    assertNull(result); // not found
  }

  @Test
  void testGetVariant() throws Exception {
    engine.takeState(simpleFeatures);

    Context context = new Context();
    VariantDef variant = engine.getVariant("Feature.A", context).value;

    if (variant == null) {
      variant =
          new VariantDef("disabled", null, false, engine.isEnabled("Feature.A", context).value);
    }

    assertEquals("disabled", variant.getName());
    assertFalse(variant.isEnabled());
  }

  @Test
  void testGetVariantWithCustomStrategy() throws Exception {
    engine.takeState(
        "{\"version\":1,\"features\":[{\"name\":\"Feature.D\",\"description\":\"Has a custom strategy\",\"enabled\":true,\"strategies\":[{\"name\":\"custom\",\"constraints\":[],\"parameters\":{\"foo\":\"bar\"}}]}]}");

    Context context = new Context();
    WasmResponse<VariantDef> variant = engine.getVariant("Feature.D", context);

    assertEquals(variant.value.isFeatureEnabled(), true);
    assertFalse(variant.value.isEnabled());
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

      System.out.println("Executing test suite: " + suiteData.name + "\n");
      engine.takeState(objectMapper.writeValueAsString(suiteData.state));

      List<Map<String, Object>> tests = suiteData.tests;
      if (tests != null) {
        for (Map<String, Object> test : tests) {
          System.out.println("Running test: " + test.get("description") + "...");
          String contextJson = objectMapper.writeValueAsString(test.get("context"));
          Context context = objectMapper.readValue(contextJson, Context.class);
          String toggleName = (String) test.get("toggleName");
          boolean expectedResult = (Boolean) test.get("expectedResult");

          Boolean result = engine.isEnabled(toggleName, context).value;

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
          System.out.println("Running test: " + test.get("description") + "...");
          String contextJson = objectMapper.writeValueAsString(test.get("context"));
          Context context = objectMapper.readValue(contextJson, Context.class);
          String toggleName = (String) test.get("toggleName");

          VariantDef expectedResult =
              objectMapper.convertValue(test.get("expectedResult"), VariantDef.class);
          VariantDef result = engine.getVariant(toggleName, context).value;
          if (result == null) {
            // this behavior should be implemented in the SDK
            result =
                new VariantDef(
                    "disabled", null, false, engine.isEnabled(toggleName, context).value);
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
    }
  }

  @Test
  void testMetrics() throws YggdrasilError, YggdrasilInvalidInputException {
    String features = "{\"version\":1,\"features\":[" +
            "{\"name\":\"Feature.Variants.A\",\"enabled\":true,\"strategies\":[],\"variants\":[{\"name\":\"variant1\",\"weight\":1}]}," +
            "{\"name\":\"Feature.Variants.B\",\"enabled\":true,\"strategies\":[{\"name\":\"userWithId\",\"parameters\":{\"userIds\":\"123\"}}],\"variants\":[]}" +
            "]}";
    engine.takeState(features);

    engine.getVariant("Feature.Variants.A", new Context());
    engine.getVariant("Feature.Variants.B", new Context());
    engine.getVariant("Missing.but.checked", new Context());
    engine.getVariant("Missing.but.checked", new Context());

    // engine.countToggle("Feature.C", false);
    MetricsBucket bucket = engine.getMetrics();

    assertNotNull(bucket);

    Instant start = bucket.getStart();
    Instant stop = bucket.getStop();
    assertNotNull(start);
    assertNotNull(stop);
    assertTrue(!stop.isBefore(start)); // being equal is fine, being before signals some corruption
    assertTrue(start.until(Instant.now(), ChronoUnit.SECONDS) < 10); // should be within 10
    // seconds of now

    assertEquals(3, bucket.getToggles().size());

    assertEquals(1, bucket.getToggles().get("Feature.Variants.A").getVariants().get("variant1"));
    assertNull(bucket.getToggles().get("Missing.Feature"));

    assertEquals(0, bucket.getToggles().get("Feature.Variants.B").getYes());
    assertEquals(1, bucket.getToggles().get("Feature.Variants.B").getNo());

    assertEquals(0, bucket.getToggles().get("Missing.but.checked").getYes());
    assertEquals(2, bucket.getToggles().get("Missing.but.checked").getNo());
  }

  @ParameterizedTest
  @CsvSource({
    "with.impression.data, true",
    "with.impression.data.false, false",
    "with.impression.data.undefined, false"
  })
  void impressionData_whenFeature_shouldReturn(String featureName, boolean expectedImpressionData)
      throws Exception {

    takeFeaturesFromResource(engine, "impression-data-tests.json");
    WasmResponse<Boolean> result = engine.isEnabled(featureName, new Context());
    assertNotNull(result);
    assertEquals(expectedImpressionData, result.impressionData);
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
    Boolean result = customEngine.isEnabled(featureName, context).value;
    assertNotNull(result);
    assertEquals(expectedIsEnabled, result);
  }

  @SuppressWarnings("unused")
  @Test
  void testResourceCleanup() throws InterruptedException {
    int mockPointer = 1;

    NativeInterface wasmInterface = Mockito.mock(NativeInterface.class);

    Mockito.when(wasmInterface.newEngine(Mockito.anyLong())).thenReturn(mockPointer);

    ReferenceQueue<UnleashEngine> queue = new ReferenceQueue<>();

    UnleashEngine library = new UnleashEngine(null, null, wasmInterface);
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
    Mockito.verify(wasmInterface).freeEngine(mockPointer);
  }

  @Test
  void testBuiltInStrategiesAreRetrieved() {
    List<String> strategies = UnleashEngine.getBuiltInStrategies();

    assertNotNull(strategies);
    assertFalse(strategies.isEmpty());
    assertTrue(strategies.contains("default"));
    assertTrue(strategies.contains("userWithId"));
    assertTrue(strategies.contains("gradualRolloutUserId"));
    assertTrue(strategies.contains("gradualRolloutRandom"));
    assertTrue(strategies.contains("applicationHostname"));
    assertTrue(strategies.contains("gradualRolloutSessionId"));
    assertTrue(strategies.contains("remoteAddress"));
    assertTrue(strategies.contains("flexibleRollout"));
  }

  @Test
  void testCoreVersionIsRetrieved() {
    String coreVersion = UnleashEngine.getCoreVersion();
    assertNotNull(coreVersion);
    // check that it contains two dots, close enough for a quick and dirty but
    // stable semver check
    assertTrue(coreVersion.split("\\.").length >= 3);
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

  @Test
  public void getMetricsReturnsCorrectResult() throws Exception {
    UnleashEngine engine = new UnleashEngine();
    String path = "../test-data/simple.json";
    String json = new String(java.nio.file.Files.readAllBytes(java.nio.file.Paths.get(path)));
    engine.takeState(json);
    engine.isEnabled("Feature.A", new Context());
    engine.isEnabled("Feature.C", new Context());
    engine.isEnabled("Feature.C", new Context());
    MetricsBucket bucket = engine.getMetrics();
    FeatureCount featA = bucket.getToggles().get("Feature.A");
    FeatureCount featC = bucket.getToggles().get("Feature.C");
    assert (featA.getYes() == 1);
    assert (featC.getYes() == 2);
  }

  @Test
  public void metricsBucketStartStopAreCorrect() throws Exception {
    UnleashEngine engine = new UnleashEngine();
    String path = "../test-data/simple.json";
    String json = new String(java.nio.file.Files.readAllBytes(java.nio.file.Paths.get(path)));
    engine.takeState(json);
    engine.isEnabled("Feature.A", new Context());
    MetricsBucket bucket = engine.getMetrics();

    ZonedDateTime now = ZonedDateTime.now(ZoneOffset.UTC);

    Instant start = bucket.getStart();
    ZonedDateTime utcStart = start.atZone(ZoneOffset.UTC);

    Instant stop = bucket.getStop();
    ZonedDateTime utcStop = stop.atZone(ZoneOffset.UTC);

    assert (utcStart.isBefore(now))
        : "start not before now. start: " + utcStart + " - stop: " + utcStop;
    assert (utcStart.plusMinutes(1).isAfter(now))
        : "start plus minute not after now. start: " + utcStart + " - stop: " + utcStop;
    assert (utcStop.isBefore(now)) : "stop not before now";
    assert (utcStop.plusMinutes(1).isAfter(now)) : "stop plus minute not after now" + utcStop;
  }

  @Test
  public void getEmptyMetricsBucketReturnsNull() throws Exception {
    UnleashEngine engine = new UnleashEngine();
    String path = "../test-data/simple.json";
    String json = new String(java.nio.file.Files.readAllBytes(java.nio.file.Paths.get(path)));
    engine.takeState(json);
    MetricsBucket bucket = engine.getMetrics();
    assert (bucket == null);
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

    for (int i = 0; i < 2; i++) {
      new Thread(
              () -> {
                try {
                  for (int j = 0; j < 1000; j++) {
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
    latch.await();
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
