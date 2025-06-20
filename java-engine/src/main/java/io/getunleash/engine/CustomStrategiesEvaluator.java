package io.getunleash.engine;

import static java.util.function.Function.identity;
import static java.util.stream.Collectors.toMap;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.util.*;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

class CustomStrategiesEvaluator {
  private static final Logger log = LoggerFactory.getLogger(CustomStrategiesEvaluator.class);
  static final Map<String, Boolean> EMPTY_STRATEGY_RESULTS = new HashMap<>();
  private final Map<String, IStrategy> registeredStrategies;
  private final Set<String> builtinStrategies;

  private final IStrategy fallbackStrategy;
  private final ObjectMapper mapper;

  private Map<String, List<MappedStrategy>> featureStrategies = new HashMap<>();

  public CustomStrategiesEvaluator(
      Stream<IStrategy> customStrategies, Set<String> builtinStrategies) {
    this(customStrategies, null, builtinStrategies);
  }

  public CustomStrategiesEvaluator(
      Stream<IStrategy> customStrategies,
      IStrategy fallbackStrategy,
      Set<String> builtinStrategies) {
    this.builtinStrategies = builtinStrategies;
    this.mapper = new ObjectMapper();
    this.mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
    this.registeredStrategies =
        customStrategies.collect(toMap(IStrategy::getName, identity(), (a, b) -> a));
    this.fallbackStrategy = fallbackStrategy;
  }

  public void loadStrategiesFor(String toggles) {
    if (this.registeredStrategies.isEmpty() && this.fallbackStrategy == null) {
      return;
    }

    if (toggles == null || toggles.isEmpty()) {
      return;
    }

    try {
      VersionedFeatures wrapper =
          mapper.readValue(toggles, new TypeReference<VersionedFeatures>() {});
      if (wrapper.features != null) {
        featureStrategies =
            wrapper.features.stream()
                .collect(toMap(feature -> feature.name, this::getFeatureStrategies));
      }
    } catch (JsonProcessingException e) {
      log.warn(
          "Error processing features. This means custom strategies will return false every time they're used",
          e);
    }
  }

  List<MappedStrategy> getFeatureStrategies(FeatureDefinition feature) {
    List<MappedStrategy> mappedStrategies = new ArrayList<>();
    int index = 1;
    for (StrategyDefinition strategyDefinition : feature.strategies) {
      if (builtinStrategies.contains(strategyDefinition.name)) {
        continue;
      }
      IStrategy impl =
          Optional.ofNullable(registeredStrategies.get(strategyDefinition.name))
              .orElseGet(() -> alwaysFalseStrategy(strategyDefinition.name));
      mappedStrategies.add(
          new MappedStrategy("customStrategy" + (index++), impl, strategyDefinition));
    }
    if (fallbackStrategy != null) {
      mappedStrategies.add(
          new MappedStrategy(
              "customStrategy" + index,
              fallbackStrategy,
              new StrategyDefinition("fallback", Collections.emptyMap())));
    }
    return mappedStrategies;
  }

  public Map<String, Boolean> eval(String name, Context context) {

    List<MappedStrategy> mappedStrategies = featureStrategies.get(name);
    if (mappedStrategies == null || mappedStrategies.isEmpty()) {
      return Collections.emptyMap();
    }

    Map<String, Boolean> results =
        mappedStrategies.stream()
            .collect(
                Collectors.toMap(
                    mappedStrategy -> mappedStrategy.resultName,
                    mappedStrategy -> tryIsEnabled(context, mappedStrategy).orElse(false)));

    return results;
  }

  private static Optional<Boolean> tryIsEnabled(Context context, MappedStrategy mappedStrategy) {
    try {
      return Optional.of(
          mappedStrategy.implementation.isEnabled(
              mappedStrategy.strategyDefinition.parameters, context));
    } catch (Exception e) {
      log.warn("Error evaluating custom strategy {}", mappedStrategy.strategyDefinition.name, e);
      return Optional.empty();
    }
  }

  private static class VersionedFeatures {
    private final List<FeatureDefinition> features;

    @JsonCreator
    private VersionedFeatures(@JsonProperty("features") List<FeatureDefinition> features) {
      this.features = features;
    }
  }

  static class FeatureDefinition {
    private final String name;
    private final List<StrategyDefinition> strategies;

    @JsonCreator
    FeatureDefinition(
        @JsonProperty("name") String name,
        @JsonProperty("strategies") List<StrategyDefinition> strategies) {
      this.name = name;
      this.strategies = strategies;
    }
  }

  static class StrategyDefinition {
    private final String name;
    private final Map<String, String> parameters;

    @JsonCreator
    StrategyDefinition(
        @JsonProperty("name") String name,
        @JsonProperty("parameters") Map<String, String> parameters) {
      this.name = name;
      this.parameters = parameters;
    }
  }

  private IStrategy alwaysFalseStrategy(String name) {
    log.warn("Custom strategy {} not found. This means it will always return false", name);
    return new IStrategy() {
      @Override
      public String getName() {
        return name;
      }

      @Override
      public boolean isEnabled(Map<String, String> parameters, Context context) {
        return false;
      }
    };
  }

  static class MappedStrategy {
    private final String resultName;
    private final IStrategy implementation;
    private final StrategyDefinition strategyDefinition;

    private MappedStrategy(
        String resultName, IStrategy implementation, StrategyDefinition strategyDefinition) {
      this.resultName = resultName;
      this.implementation = implementation;
      this.strategyDefinition = strategyDefinition;
    }
  }
}
