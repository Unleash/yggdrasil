package io.getunleash.engine;

import static io.getunleash.engine.CustomStrategies.EMPTY_STRATEGY_RESULTS;
import static io.getunleash.engine.UnleashEngineTest.readResource;
import static org.junit.jupiter.api.Assertions.*;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.IOException;
import java.net.URISyntaxException;
import java.util.Map;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;
import org.junit.jupiter.params.provider.NullSource;
import org.junit.jupiter.params.provider.ValueSource;

class CustomStrategiesTest {

    private static final IStrategy TEST_STRATEGY =
            new IStrategy() {
                @Override
                public String getName() {
                    return "test-strategy";
                }

                @Override
                public boolean isEnabled(Map<String, String> parameters, Context context) {
                    return true;
                }
            };

    public static Stream<Arguments> invalidNamesAndContext() {
        return Stream.of(
                Arguments.of(Stream.empty(), null, null),
                Arguments.of(Stream.empty(), null, new Context()),
                Arguments.of(Stream.empty(), "", null),
                Arguments.of(Stream.empty(), "", new Context()),
                Arguments.of(Stream.of(TEST_STRATEGY), null, null),
                Arguments.of(Stream.of(TEST_STRATEGY), null, new Context()),
                Arguments.of(Stream.of(TEST_STRATEGY), "", null),
                Arguments.of(Stream.of(TEST_STRATEGY), "", new Context()));
    }

    @ParameterizedTest
    @MethodSource("invalidNamesAndContext")
    void invalidNameAndContext_shouldEvalToEmpty(
            Stream<IStrategy> strategies, String name, Context context) {
        CustomStrategies customStrategies = new CustomStrategies(strategies);
        assertEquals(EMPTY_STRATEGY_RESULTS, customStrategies.eval(name, context));
    }

    @ParameterizedTest
    @ValueSource(strings = {"", "[]", "{}", "{\"version\": 2, \"features\": []}"})
    @NullSource
    void shouldBeAbleToTakeAnyStateWithoutFailing(String state) {
        CustomStrategies customStrategies = new CustomStrategies(Stream.of(TEST_STRATEGY));
        assertDoesNotThrow(() -> customStrategies.takeState(state));
    }

    @Test
    void singleUnknownStrategy_shouldEvalToEmpty() throws IOException, URISyntaxException {
        CustomStrategies customStrategies = new CustomStrategies(Stream.of(TEST_STRATEGY));
        customStrategies.takeState(readResource("custom-strategy-tests.json"));
        assertSameObjects(
                EMPTY_STRATEGY_RESULTS,
                customStrategies.eval("Feature.Custom.Strategies", new Context()));
    }

    @Test
    void singleExistingStrategy_shouldEvalToOneStrategy() throws IOException, URISyntaxException {
        CustomStrategies customStrategies = new CustomStrategies(Stream.of(alwaysTrue("custom")));
        customStrategies.takeState(readResource("custom-strategy-tests.json"));
        assertSameObjects(
                "{\"customStrategy1\":true}",
                customStrategies.eval("Feature.Custom.Strategies", new Context()));
    }

    @Test
    void twoExistingStrategy_shouldEvalToBothStrategies() throws IOException, URISyntaxException {
        CustomStrategies customStrategies =
                new CustomStrategies(Stream.of(alwaysTrue("custom"), alwaysTrue("cus-tom")));
        customStrategies.takeState(readResource("custom-strategy-tests.json"));
        assertSameObjects(
                "{\"customStrategy1\":true,\"customStrategy2\":true}",
                customStrategies.eval("Feature.Custom.Strategies", new Context()));
    }

    private void assertSameObjects(String expected, String result) {
        ObjectMapper objectMapper = new ObjectMapper();
        Map<String, Boolean> expectedMap =
                assertDoesNotThrow(
                        () -> objectMapper.readValue(expected, new TypeReference<>() {}));
        Map<String, Boolean> resultMap =
                assertDoesNotThrow(() -> objectMapper.readValue(result, new TypeReference<>() {}));
        assertEquals(expectedMap, resultMap);
    }

    private IStrategy alwaysTrue(String name) {
        return new IStrategy() {
            @Override
            public String getName() {
                return name;
            }

            @Override
            public boolean isEnabled(Map<String, String> parameters, Context context) {
                return true;
            }
        };
    }
}
