from dataclasses import asdict
import json
from yggdrasil_engine.engine import UnleashEngine, Variant, FeatureDefinition
import json
import os


CUSTOM_STRATEGY_STATE = """
{
    "version": 1,
    "features": [
        {
            "name": "Feature.A",
            "enabled": true,
            "strategies": [
                {
                    "name": "breadStrategy",
                    "parameters": {}
                }
            ],
            "variants": [
                {
                    "name": "sourDough",
                    "weight": 100
                }
            ],
            "impressionData": true
        }
    ]
}
"""


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


def test_custom_strategies_work_end_to_end():
    engine = UnleashEngine()

    class BreadStrategy:
        def apply(self, _parameters, context):
            return context.get("betterThanSlicedBread") == True

    engine.register_custom_strategies({"breadStrategy": BreadStrategy()})
    engine.take_state(CUSTOM_STRATEGY_STATE)

    enabled_when_better = engine.is_enabled(
        "Feature.A", {"betterThanSlicedBread": True}
    )
    disabled_when_not_better = engine.is_enabled(
        "Feature.A", {"betterThanSlicedBread": False}
    )

    should_be_sour_dough = engine.get_variant(
        "Feature.A", {"betterThanSlicedBread": True}
    )

    assert enabled_when_better == True
    assert disabled_when_not_better == False
    assert should_be_sour_dough.name == "sourDough"


def test_increments_counts_for_yes_no_and_variants():
    engine = UnleashEngine()

    with open("../test-data/simple.json") as file:
        state = json.load(file)

    engine.take_state(json.dumps(state))

    engine.count_toggle("testToggle", True)
    engine.count_toggle("testToggle", True)
    engine.count_toggle("testToggle", False)
    engine.count_variant("testToggle", "disabled")

    metrics = engine.get_metrics()

    assert metrics["toggles"]["testToggle"]["yes"] == 2
    assert metrics["toggles"]["testToggle"]["no"] == 1
    assert metrics["toggles"]["testToggle"]["variants"]["disabled"] == 1


def test_metrics_are_flushed_when_get_metrics_is_called():
    engine = UnleashEngine()

    with open("../test-data/simple.json") as file:
        state = json.load(file)

    engine.take_state(json.dumps(state))

    engine.count_toggle("testToggle", True)

    metrics = engine.get_metrics()
    assert metrics["toggles"]["testToggle"]["yes"] == 1

    metrics = engine.get_metrics()
    assert metrics is None


def test_metrics_are_still_incremented_when_toggle_does_not_exist():
    engine = UnleashEngine()

    engine.count_toggle("aToggleSoSecretItDoesNotExist", True)

    metrics = engine.get_metrics()

    assert metrics["toggles"]["aToggleSoSecretItDoesNotExist"]["yes"] == 1


def test_yields_impression_data():
    engine = UnleashEngine()

    engine.take_state(CUSTOM_STRATEGY_STATE)

    assert engine.should_emit_impression_event("Feature.A")
    assert not engine.should_emit_impression_event("Nonexisting")


def test_list_known_toggles():
    engine = UnleashEngine()

    engine.take_state(CUSTOM_STRATEGY_STATE)
    first_toggle = engine.list_known_toggles()[0]

    assert len(engine.list_known_toggles()) == 1
    assert first_toggle == FeatureDefinition(
        name="Feature.A", project="default", type=None
    )


def test_list_empty_toggles_yields_empty_list():
    engine = UnleashEngine()

    assert engine.list_known_toggles() == []

def test_get_state_and_roundtrip():
    """Test get_state returns valid JSON and supports roundtrip"""
    engine = UnleashEngine()

    empty_state = engine.get_state()
    assert '"features": []' in empty_state
    assert 'status_code' not in empty_state
    assert 'error_message' not in empty_state
    
    test_state = {
        "version": 1,
        "features": [{"name": "testFeature", "enabled": True, "strategies": [{"name": "default"}]}]
    }

    engine.take_state(json.dumps(test_state))
    retrieved_state = engine.get_state()

    assert "testFeature" in retrieved_state
    assert '"version": 1' in retrieved_state
    assert '"name": "testFeature"' in retrieved_state
    assert '"name": "default"' in retrieved_state
    assert 'status_code' not in retrieved_state
    assert 'error_message' not in retrieved_state