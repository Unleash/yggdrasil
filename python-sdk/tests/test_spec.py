import json
import uuid
from os import path
import pytest
from python_sdk import UnleashEngine, Context


CLIENT_SPEC_PATH = "../client-specification/specifications"


def load_spec(spec):
    with open(path.join(CLIENT_SPEC_PATH, spec)) as _f:
        data = json.load(_f)
        return (
            data["name"],
            data["state"],
            data.get("tests") or [],
            data.get("variantTests") or [],
        )


def load_specs():
    with open(path.join(CLIENT_SPEC_PATH, "index.json")) as _f:
        return json.load(_f)


def iter_spec():
    count = 0
    for spec in load_specs():
        count += 1
        if count > 5:
            break
        name, state, tests, variant_tests = load_spec(spec)

        for test in tests:
            yield name, test["description"], state, test, False

        for variant_test in variant_tests:
            yield name, test["description"], state, variant_test, True


try:
    ALL_SPECS = list(iter_spec())
    TEST_DATA = [x[2:] for x in ALL_SPECS]
    TEST_NAMES = [f"{x[0]}-{x[1]}" for x in ALL_SPECS]
except FileNotFoundError:
    print(
        "Cannot find the client specifications, these can be downloaded with the following command: 'git clone --depth 5 --branch v4.2.2 https://github.com/Unleash/client-specification.git tests/specification_tests/client-specification'"
    )
    raise


def to_context(context):
    user_id = context.get("userId")
    session_id = context.get("sessionId")
    environment = context.get("sessionId")
    app_name = context.get("sessionId")
    remote_address = context.get("remote_address")
    properties = context.get("properties")
    return Context(
        user_id, session_id, remote_address, environment, app_name, properties
    )


@pytest.mark.parametrize("spec", TEST_DATA, ids=TEST_NAMES)
def test_spec(spec):
    unleash_engine = UnleashEngine()
    state, test_data, is_variant_test = spec
    unleash_engine.take_state(json.dumps(state))
    context = to_context(test_data.get("context"))
    if not is_variant_test:
        toggle_name = test_data["toggleName"]
        expected = test_data["expectedResult"]
        assert unleash_engine.is_enabled(toggle_name, context) == expected
    else:
        toggle_name = test_data["toggleName"]
        expected = test_data["expectedResult"]

        variant = unleash_engine.get_variant(toggle_name, context)
        assert variant == expected
