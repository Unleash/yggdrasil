package io.getunleash.engine;

import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

class CamelToSnakeMapperTest {

    @ParameterizedTest
    @CsvSource({
            "snakesAreTotallyCoolerThanCamels, snakes_are_totally_cooler_than_camels",
            "single, single",
            "somethingWithASingleCharacter, something_with_a_single_character",
    })
    void testComplexSnakeNameIsConvertedToCamel(String input, String expected) {
        CamelToSnakeMapper mapper = new CamelToSnakeMapper();
        String snakeName = mapper.convertToSnake(input);
        assert (snakeName.equals(expected));
    }
}