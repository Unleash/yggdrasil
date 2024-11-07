import ctypes
import json
import os
import platform
from contextlib import contextmanager
from enum import Enum
from dataclasses import dataclass, field
from typing import Any, Callable, ClassVar, Dict, List, Optional, Type, TypeVar, cast


def _get_binary_path():
    lib_dir = os.path.join(os.path.dirname(__file__), "lib")
    system = platform.system()

    if system == "Linux":
        return os.path.join(lib_dir, "libyggdrasilffi.so")
    elif system == "Darwin":
        return os.path.join(lib_dir, "libyggdrasilffi.dylib")
    elif system == "Windows":
        return os.path.join(lib_dir, "yggdrasilffi.so.dll")
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
class Response:
    status_code: StatusCode
    value: Optional[any]
    error_message: Optional[str]

    # this only exists to handle feature_enabled/featureEnabled
    deserializers: ClassVar[Dict[Type, Callable[[Any], Any]]] = {
        Variant: Variant.from_dict,
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

        self.state = self.lib.new_engine()

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
        with self.materialize_pointer(response_ptr, List[Warning]) as result:
            if result.value:
                warnings = "\n".join(
                    [f"{warning.toggle_name}: {warning.message}" for warning in result]
                )
                return warnings
            return None

    def is_enabled(self, toggle_name: str, context: dict) -> Optional[bool]:
        serialized_context = json.dumps(context or {})
        custom_strategy_results = json.dumps({})

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
        custom_strategy_results = json.dumps({})

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
