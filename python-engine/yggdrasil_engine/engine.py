import ctypes
import json
import os
import platform
from contextlib import contextmanager
from enum import Enum
from dataclasses import dataclass, field
from typing import Any, Callable, ClassVar, Dict, List, Optional, Type, TypeVar, cast
from yggdrasil_engine.custom_strategy import CustomStrategyHandler


def _get_binary_path():
    lib_dir = os.path.join(os.path.dirname(__file__), "lib")
    system = platform.system()

    if system == "Linux":
        return os.path.join(lib_dir, "libyggdrasilffi.so")
    elif system == "Darwin":
        return os.path.join(lib_dir, "libyggdrasilffi.dylib")
    elif system == "Windows":
        return os.path.join(lib_dir, "yggdrasilffi.dll")
    else:
        raise RuntimeError(f"Unsupported operating system: {system}")


T = TypeVar("T")


class StatusCode(Enum):
    OK = "Ok"
    NOT_FOUND = "NotFound"
    ERROR = "Error"


class YggdrasilError(Exception):
    pass


@dataclass
class Variant:
    name: str
    payload: Optional[Dict[str, str]] = field(default_factory=dict)
    enabled: bool = False
    feature_enabled: bool = False

    @staticmethod
    def from_dict(data: dict) -> "Variant":
        return Variant(
            name=data.get("name", ""),
            payload=data.get("payload"),
            enabled=data.get("enabled", False),
            feature_enabled=data.get("featureEnabled", False),
        )


@dataclass
class FeatureDefinition:
    name: str
    project: str
    type: Optional[str]

    @staticmethod
    def from_dict(data: dict) -> "FeatureDefinition":
        return FeatureDefinition(
            name=data.get("name", ""),
            project=data.get("project", ""),
            type=data.get("type"),
        )


def load_feature_defs(raw_defs: List[dict]) -> List[FeatureDefinition]:
    return [FeatureDefinition.from_dict(defn) for defn in raw_defs]


@dataclass
class Response:
    status_code: StatusCode
    value: Optional[any]
    error_message: Optional[str]

    deserializers: ClassVar[Dict[Type, Callable[[Any], Any]]] = {
        Variant: Variant.from_dict,
        List[FeatureDefinition]: load_feature_defs,
    }

    @staticmethod
    def from_json(data: str, value_type: Type[T]) -> "Response[T]":
        status_code = StatusCode(data["status_code"])
        raw_value = data.get("value")
        error_message = data.get("error_message")

        if raw_value is not None:
            if value_type in Response.deserializers:
                value = Response.deserializers[value_type](raw_value)
            else:
                value = cast(value_type, raw_value)
        else:
            value = None

        return Response(
            status_code=status_code, value=value, error_message=error_message
        )


class UnleashEngine:
    def __init__(self):
        binary_path = _get_binary_path()

        self.lib = ctypes.CDLL(binary_path)
        self.lib.new_engine.restype = ctypes.c_void_p
        self.lib.take_state.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self.lib.take_state.restype = ctypes.POINTER(ctypes.c_char)
        self.lib.check_enabled.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]
        self.lib.check_enabled.restype = ctypes.POINTER(ctypes.c_char)
        self.lib.check_variant.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]
        self.lib.check_variant.restype = ctypes.POINTER(ctypes.c_char)
        self.lib.free_engine.argtypes = [ctypes.c_void_p]
        self.lib.free_engine.restype = None
        self.lib.free_response.argtypes = [ctypes.c_void_p]
        self.lib.free_response.restype = None

        self.lib.get_metrics.argtypes = [ctypes.c_void_p]
        self.lib.get_metrics.restype = ctypes.POINTER(ctypes.c_char)

        self.lib.get_state.argtypes = [ctypes.c_void_p]
        self.lib.get_state.restype = ctypes.POINTER(ctypes.c_char)

        self.lib.count_toggle.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_bool,
        ]
        self.lib.count_toggle.restype = ctypes.POINTER(ctypes.c_char)

        self.lib.count_variant.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]
        self.lib.count_variant.restype = ctypes.POINTER(ctypes.c_char)

        self.lib.should_emit_impression_event.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
        ]

        self.lib.should_emit_impression_event.restype = ctypes.POINTER(ctypes.c_char)

        self.lib.list_known_toggles.argtypes = [ctypes.c_void_p]
        self.lib.list_known_toggles.restype = ctypes.POINTER(ctypes.c_char)

        self.state = self.lib.new_engine()
        self.custom_strategy_handler = CustomStrategyHandler()

    def __del__(self):
        if hasattr(self, "state") and self.state is not None:
            self.lib.free_engine(self.state)

    @contextmanager
    def materialize_pointer(self, ptr, value_type: Type[T]):
        try:
            response = ctypes.cast(ptr, ctypes.c_char_p).value.decode("utf-8")
            yield Response.from_json(json.loads(response), value_type)
        finally:
            self.lib.free_response(ptr)

    def take_state(self, state_json: str) -> Optional[List[Warning]]:
        response_ptr = self.lib.take_state(self.state, state_json.encode("utf-8"))
        self.custom_strategy_handler.update_strategies(state_json)
        with self.materialize_pointer(response_ptr, List[Warning]) as result:
            if result.value:
                warnings = "\n".join(
                    [f"{warning.toggle_name}: {warning.message}" for warning in result]
                )
                return warnings
            return None

    def get_state(self) -> str:
        response_ptr = self.lib.get_state(self.state)
        with self.materialize_pointer(response_ptr, dict) as result:
            return json.dumps(result.value)

    def is_enabled(self, toggle_name: str, context: dict) -> Optional[bool]:
        serialized_context = json.dumps(context or {})
        custom_strategy_results = json.dumps(
            self.custom_strategy_handler.evaluate_custom_strategies(
                toggle_name, context
            )
        )

        response_ptr = self.lib.check_enabled(
            self.state,
            toggle_name.encode("utf-8"),
            serialized_context.encode("utf-8"),
            custom_strategy_results.encode("utf-8"),
        )
        with self.materialize_pointer(response_ptr, bool) as response:
            if response.status_code == StatusCode.ERROR:
                raise YggdrasilError(response.error_message)
            return response.value

    def get_variant(self, toggle_name: str, context: dict) -> Optional[Variant]:
        serialized_context = json.dumps(context or {})
        custom_strategy_results = json.dumps(
            self.custom_strategy_handler.evaluate_custom_strategies(
                toggle_name, context
            )
        )

        response_ptr = self.lib.check_variant(
            self.state,
            toggle_name.encode("utf-8"),
            serialized_context.encode("utf-8"),
            custom_strategy_results.encode("utf-8"),
        )
        with self.materialize_pointer(response_ptr, Variant) as response:
            if response.status_code == StatusCode.ERROR:
                raise YggdrasilError(response.error_message)
            return response.value

    def register_custom_strategies(self, custom_strategies: dict):
        self.custom_strategy_handler.register_custom_strategies(custom_strategies)

    def count_toggle(self, toggle_name: str, enabled: bool):
        response_ptr = self.lib.count_toggle(
            self.state, toggle_name.encode("utf-8"), enabled
        )
        self.lib.free_response(response_ptr)

    def count_variant(self, toggle_name: str, variant_name: str):
        response_ptr = self.lib.count_variant(
            self.state, toggle_name.encode("utf-8"), variant_name.encode("utf-8")
        )
        self.lib.free_response(response_ptr)

    def get_metrics(self) -> Dict[str, Any]:
        metrics_ptr = self.lib.get_metrics(self.state)
        with self.materialize_pointer(metrics_ptr, Dict[str, Any]) as response:
            if response.status_code == StatusCode.ERROR:
                raise YggdrasilError(response.error_message)
            return response.value

    def should_emit_impression_event(self, toggle_name: str) -> bool:
        response_ptr = self.lib.should_emit_impression_event(
            self.state, toggle_name.encode("utf-8")
        )
        with self.materialize_pointer(response_ptr, bool) as response:
            if response.status_code == StatusCode.ERROR:
                raise YggdrasilError(response.error_message)
            return response.value

    def list_known_toggles(self) -> List[FeatureDefinition]:
        response_ptr = self.lib.list_known_toggles(self.state)
        with self.materialize_pointer(
            response_ptr, List[FeatureDefinition]
        ) as response:
            if response.status_code == StatusCode.ERROR:
                raise YggdrasilError(response.error_message)
            return response.value
