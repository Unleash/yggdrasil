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

class CustomStrategies {
    private static final Logger log = LoggerFactory.getLogger(CustomStrategies.class);
    static final String EMPTY_STRATEGY_RESULTS = "{}";
    private final Map<String, IStrategy> registeredStrategies;
    private final ObjectMapper mapper;

    private Map<String, List<MappedStrategy>> featureStrategies = new HashMap<>();

    public CustomStrategies(Stream<IStrategy> customStrategies) {
        this.mapper = new ObjectMapper();
        this.mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
        this.registeredStrategies = customStrategies.collect(toMap(IStrategy::getName, identity()));
    }

    public void takeState(String toggles) {
        if (this.registeredStrategies.isEmpty()) {
            return;
        }

        if (toggles == null || toggles.isEmpty()) {
            return;
        }

        try {
            VersionedFeatures wrapper = mapper.readValue(toggles, new TypeReference<>() {});
            if (wrapper.features != null) {
                featureStrategies =
                        wrapper.features.stream()
                                .collect(
                                        toMap(feature -> feature.name, this::getFeatureStrategies));
            }
        } catch (JsonProcessingException e) {
            log.warn(
                    "Error processing features. This means custom strategies will return false every time they're used",
                    e);
        }
    }

    private List<MappedStrategy> getFeatureStrategies(FeatureDefinition feature) {
        List<MappedStrategy> mappedStrategies = new ArrayList<>();
        int index = 1;
        for (StrategyDefinition strategyDefinition : feature.strategies) {
            IStrategy impl = registeredStrategies.get(strategyDefinition.name);
            if (impl == null) {
                log.warn(
                        "Custom strategy {} not found. This means it will always return false",
                        strategyDefinition.name);
            } else {
                mappedStrategies.add(
                        new MappedStrategy("customStrategy" + (index++), impl, strategyDefinition));
            }
        }
        return mappedStrategies;
    }

    public String eval(String name, Context context) {

        List<MappedStrategy> mappedStrategies = featureStrategies.get(name);

        if (mappedStrategies == null) {
            return EMPTY_STRATEGY_RESULTS;
        }

        Map<String, Boolean> results =
                mappedStrategies.stream()
                        .collect(
                                Collectors.toMap(
                                        mappedStrategy -> mappedStrategy.resultName,
                                        mappedStrategy ->
                                                mappedStrategy.implementation.isEnabled(
                                                        mappedStrategy
                                                                .strategyDefinition
                                                                .parameters,
                                                        context)));
        try {
            return mapper.writeValueAsString(results);
        } catch (JsonProcessingException e) {
            log.warn(
                    "Error processing strategy results. This means custom strategies will return false every time they're used",
                    e);
            return EMPTY_STRATEGY_RESULTS;
        }
    }

    private static class VersionedFeatures {
        private final List<FeatureDefinition> features;

        @JsonCreator
        private VersionedFeatures(@JsonProperty("features") List<FeatureDefinition> features) {
            this.features = features;
        }
    }

    private static class FeatureDefinition {
        private final String name;
        private final List<StrategyDefinition> strategies;

        @JsonCreator
        private FeatureDefinition(
                @JsonProperty("name") String name,
                @JsonProperty("strategies") List<StrategyDefinition> strategies) {
            this.name = name;
            this.strategies = strategies;
        }
    }

    private static class StrategyDefinition {
        private final String name;
        private final Map<String, String> parameters;

        @JsonCreator
        private StrategyDefinition(
                @JsonProperty("name") String name,
                @JsonProperty("parameters") Map<String, String> parameters) {
            this.name = name;
            this.parameters = parameters;
        }
    }

    private static class MappedStrategy {
        private final String resultName;
        private final IStrategy implementation;
        private final StrategyDefinition strategyDefinition;

        private MappedStrategy(
                String resultName,
                IStrategy implementation,
                StrategyDefinition strategyDefinition) {
            this.resultName = resultName;
            this.implementation = implementation;
            this.strategyDefinition = strategyDefinition;
        }
    }
}
