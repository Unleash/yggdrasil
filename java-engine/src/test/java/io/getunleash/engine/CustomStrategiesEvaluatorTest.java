package io.getunleash.engine;

import static io.getunleash.engine.CustomStrategiesEvaluator.EMPTY_STRATEGY_RESULTS;
import static io.getunleash.engine.TestStrategies.alwaysFails;
import static io.getunleash.engine.TestStrategies.alwaysTrue;
import static io.getunleash.engine.UnleashEngineTest.readResource;
import static org.junit.jupiter.api.Assertions.*;
import static org.junit.jupiter.params.provider.Arguments.of;

import io.getunleash.engine.CustomStrategiesEvaluator.FeatureDefinition;
import io.getunleash.engine.CustomStrategiesEvaluator.MappedStrategy;
import io.getunleash.engine.CustomStrategiesEvaluator.StrategyDefinition;
import java.io.IOException;
import java.net.URISyntaxException;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.*;

class CustomStrategiesEvaluatorTest {

  public static Stream<Arguments> invalidNamesAndContext() {
    IStrategy testStrategy = alwaysTrue("test-strategy");
    return Stream.of(
        of(Stream.empty(), null, null),
        of(Stream.empty(), null, new Context()),
        of(Stream.empty(), "", null),
        of(Stream.empty(), "", new Context()),
        of(Stream.of(testStrategy), null, null),
        of(Stream.of(testStrategy), null, new Context()),
        of(Stream.of(testStrategy), "", null),
        of(Stream.of(testStrategy), "", new Context()));
  }

  private static Stream<Arguments> twoStrategies() {
    return Stream.of(
        of(
            alwaysTrue("custom"),
            alwaysTrue("cus-tom"),
            Map.of("customStrategy1", true, "customStrategy2", true)),
        of(
            alwaysFails("custom"),
            alwaysFails("cus-tom"),
            Map.of("customStrategy1", false, "customStrategy2", false)),
        of(
            alwaysTrue("custom"),
            alwaysFails("cus-tom"),
            Map.of("customStrategy1", true, "customStrategy2", false)),
        of(
            alwaysFails("custom"),
            alwaysTrue("cus-tom"),
            Map.of("customStrategy1", false, "customStrategy2", true)),
        of(
            alwaysTrue("wrongName"),
            alwaysTrue("wrongName"),
            Map.of("customStrategy1", false, "customStrategy2", false)),
        of(
            alwaysTrue("custom"),
            alwaysTrue("custom"),
            Map.of("customStrategy1", true, "customStrategy2", false)));
  }

  private static Stream<Arguments> singleStrategy() {
    return Stream.of(
        of("custom", Map.of("customStrategy1", true, "customStrategy2", false)),
        of("cus-tom", Map.of("customStrategy1", false, "customStrategy2", true)),
        of("unknown", Map.of("customStrategy1", false, "customStrategy2", false)));
  }

  @ParameterizedTest
  @MethodSource("invalidNamesAndContext")
  void invalidNameAndContext_shouldEvalToEmpty(
      Stream<IStrategy> strategies, String name, Context context) {
    CustomStrategiesEvaluator customStrategiesEvaluator =
        new CustomStrategiesEvaluator(strategies, new HashSet<>());
    assertEquals(EMPTY_STRATEGY_RESULTS, customStrategiesEvaluator.eval(name, context));
  }

  @ParameterizedTest
  @ValueSource(strings = {"", "[]", "{}", "{\"version\": 2, \"features\": []}"})
  @NullSource
  void shouldBeAbleToTakeAnyStateWithoutFailing(String state) {
    CustomStrategiesEvaluator customStrategiesEvaluator =
        new CustomStrategiesEvaluator(Stream.of(alwaysTrue("test-strategy")), new HashSet<>());
    assertDoesNotThrow(() -> customStrategiesEvaluator.loadStrategiesFor(state));
  }

  @ParameterizedTest
  @MethodSource("singleStrategy")
  void singleAlwaysTrueStrategy_shouldEvalTo(String strategyName, Map<String, Boolean> expected)
      throws IOException, URISyntaxException {
    CustomStrategiesEvaluator customStrategiesEvaluator =
        new CustomStrategiesEvaluator(Stream.of(alwaysTrue(strategyName)), new HashSet<>());
    customStrategiesEvaluator.loadStrategiesFor(readResource("custom-strategy-tests.json"));
    assertEquals(
        expected, customStrategiesEvaluator.eval("Feature.Custom.Strategies", new Context()));
  }

  @ParameterizedTest
  @MethodSource("twoStrategies")
  void twoExistingStrategy_shouldEvalToBothStrategies(
      IStrategy one, IStrategy two, Map<String, Boolean> expected)
      throws IOException, URISyntaxException {
    CustomStrategiesEvaluator customStrategiesEvaluator =
        new CustomStrategiesEvaluator(Stream.of(one, two), new HashSet<>());
    customStrategiesEvaluator.loadStrategiesFor(readResource("custom-strategy-tests.json"));
    assertEquals(
        expected, customStrategiesEvaluator.eval("Feature.Custom.Strategies", new Context()));
  }

  @Test
  void faultyStrategy_shouldEvalToEmpty() throws IOException, URISyntaxException {
    CustomStrategiesEvaluator customStrategiesEvaluator =
        new CustomStrategiesEvaluator(Stream.of(alwaysFails("custom")), new HashSet<>());
    customStrategiesEvaluator.loadStrategiesFor(readResource("custom-strategy-tests.json"));
    assertEquals(
        Map.of("customStrategy1", false, "customStrategy2", false),
        customStrategiesEvaluator.eval("Feature.Custom.Strategies", new Context()));
  }

  @Test
  void doesNotLoadCustomStrategyForBuiltinStrategy() throws Exception {
    CustomStrategiesEvaluator customStrategiesEvaluator =
        new CustomStrategiesEvaluator(Stream.of(alwaysFails("custom")), Set.of("flexibleRollout"));

    StrategyDefinition flexibleRollout = new StrategyDefinition("flexibleRollout", Map.of());
    FeatureDefinition feature = new FeatureDefinition("feature", List.of(flexibleRollout));

    List<MappedStrategy> results = customStrategiesEvaluator.getFeatureStrategies(feature);

    assertTrue(results.isEmpty());
  }
}
