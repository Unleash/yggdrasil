from dataclasses import asdict
import json
from unleash_engine import UnleashEngine, Variant
import json
import os


def variant_to_dict(variant) -> dict:
    print(variant)
    return {k: v for k, v in asdict(variant).items() if v is not None}


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

            result = unleash_engine.is_enabled(toggle_name, context) or False

            assert (
                result == expected_result
            ), f"Failed test '{test['description']}': expected {expected_result}, got {result}"

        for test in suite_data.get("variantTests", []):
            context = test["context"]
            toggle_name = test["toggleName"]
            expected_result = test["expectedResult"]

            result = unleash_engine.get_variant(toggle_name, context) or Variant(
                "disabled", None, False, False
            )

            ## We get away with this right now because the casing in the spec tests for feature_enabled
            ## is snake_case. At some point this is going to change to camel case and this is going to break
            expected_json = json.dumps(expected_result)
            actual_json = json.dumps(variant_to_dict(result))

            assert (
                expected_json == actual_json
            ), f"Failed test '{test['description']}': expected {expected_json}, got {actual_json}"
