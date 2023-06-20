import json
import pytest
from unleash_engine import UnleashEngine
import json
import os


def test_get_variant_does_not_crash():
    unleash_engine = UnleashEngine()

    with open("../test-data/simple.json") as file:
        state = json.load(file)
        context = {"userId": "123"}
        toggle_name = "testToggle"

        unleash_engine.take_state(json.dumps(state))
        print(unleash_engine.get_variant(toggle_name, context))


def test_client_spec():
    unleash_engine = UnleashEngine()

    with open("../client-specification/specifications/index.json", "r") as file:
        test_suites = json.load(file)

    for suite in test_suites:
        suite_path = os.path.join("../client-specification/specifications", suite)

        with open(suite_path, "r") as suite_file:
            suite_data = json.load(suite_file)

        unleash_engine.take_state(json.dumps(suite_data["state"]))

        for test in suite_data.get("tests", []):
            context = test["context"]
            toggle_name = test["toggleName"]
            expected_result = test["expectedResult"]

            result = unleash_engine.is_enabled(toggle_name, context)

            assert (
                result == expected_result
            ), f"Failed test '{test['description']}': expected {expected_result}, got {result}"

        for test in suite_data.get("variantTests", []):
            context = test["context"]
            toggle_name = test["toggleName"]
            expected_result = test["expectedResult"]

            result = unleash_engine.get_variant(toggle_name, context)

            assert (
                result == expected_result
            ), f"Failed test '{test['description']}': expected {expected_result}, got {result}"
