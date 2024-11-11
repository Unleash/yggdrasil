from yggdrasil_engine.custom_strategy import CustomStrategyHandler
import pytest

RAW_STATE = """
{
    "version": 1,
    "features": [
        {
            "name": "Feature.A",
            "enabled": true,
            "strategies": [
                {
                    "name": "default",
                    "parameters": {}
                },
                {
                    "name": "custom",
                    "parameters": {
                        "userIsAPorcupine": "yes"
                    }
                },
                {
                    "name": "some-other-custom",
                    "parameters": {
                        "userIsAPorcupine": "yes"
                    }
                }
            ]
        }
    ]
}
"""


def test_computing_strategies_respects_their_contained_logic():
    class TestStrategy:
        def apply(self, _parameters, context):
            return context.get("jimRubsBirds") == True

    handler = CustomStrategyHandler()
    handler.update_strategies(RAW_STATE)
    handler.register_custom_strategies({"custom": TestStrategy()})

    results = handler.evaluate_custom_strategies("Feature.A", {"jimRubsBirds": True})
    assert results["customStrategy1"] == True

    results = handler.evaluate_custom_strategies("Feature.A", {"jimRubsBirds": "What?"})
    assert results["customStrategy1"] == False


def test_returns_a_result_for_every_strategy_registered():
    class TestStrategy:
        def apply(self, _parameters, _context):
            return True

    handler = CustomStrategyHandler()
    handler.update_strategies(RAW_STATE)
    handler.register_custom_strategies(
        {"custom": TestStrategy(), "some-other-custom": TestStrategy()}
    )

    results = handler.evaluate_custom_strategies("Feature.A", {})
    assert results["customStrategy1"] == True
    assert results["customStrategy2"] == True


def test_returns_false_for_custom_strategies_not_registered():
    class TestStrategy:
        def apply(self, parameters, _context):
            return parameters.get("userIsAPorcupine") == "yes"

    handler = CustomStrategyHandler()
    handler.update_strategies(RAW_STATE)
    handler.register_custom_strategies({"custom": TestStrategy()})

    results = handler.evaluate_custom_strategies("Feature.A", {})
    assert results["customStrategy1"] == True
    assert results["customStrategy2"] == False


def test_register_custom_strategy_rejects_strategies_without_apply_method():
    class IncompleteStrategy:
        pass

    handler = CustomStrategyHandler()

    with pytest.raises(ValueError):
        handler.register_custom_strategies({"custom": IncompleteStrategy()})


def test_register_custom_strategy_rejects_strategies_with_incorrect_number_of_args():
    class FirstBearStrategy:
        def apply(self, _parameters):
            return True

    class SecondBearStrategy:
        def apply(self, _parameters, _context, _extra):
            return True

    class ThirdBearStrategy:
        def apply(self, _parameters, _context):
            return True

    handler = CustomStrategyHandler()

    with pytest.raises(ValueError):
        handler.register_custom_strategies({"custom": FirstBearStrategy()})

    with pytest.raises(ValueError):
        handler.register_custom_strategies({"custom": SecondBearStrategy()})

    ## Goldilocks parameters, we expect this not to raise
    handler.register_custom_strategies({"custom": ThirdBearStrategy()})
