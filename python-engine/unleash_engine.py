import ctypes
import json
import os


class UnleashEngine:
    def __init__(self):
        lib_path = os.environ.get("YGGDRASIL_LIB_PATH")
        if lib_path is None:
            raise Exception("YGGDRASIL_LIB_PATH not set")
        combined_path = os.path.join(lib_path, "libyggdrasilffi.so")

        self.lib = ctypes.CDLL(combined_path)
        self.lib.engine_new.restype = ctypes.c_void_p
        self.lib.engine_take_state.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self.lib.engine_take_state.restype = ctypes.c_char_p
        self.lib.engine_is_enabled.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]
        self.lib.engine_is_enabled.restype = ctypes.c_bool
        self.lib.engine_get_variant.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]
        self.lib.engine_get_variant.restype = ctypes.c_char_p
        self.lib.engine_free.argtypes = [ctypes.c_void_p]
        self.lib.engine_free.restype = None

        self.state = self.lib.engine_new()

    def __del__(self):
        if hasattr(self, "state") and self.state is not None:
            self.lib.engine_free(self.state)

    def take_state(self, state_json):
        result_ptr = self.lib.engine_take_state(self.state, state_json.encode("utf-8"))
        return result_ptr.decode("utf-8")

    def is_enabled(self, toggle_name, context):
        serialized_context = json.dumps(context)
        return self.lib.engine_is_enabled(
            self.state, toggle_name.encode("utf-8"), serialized_context.encode("utf-8")
        )

    def get_variant(self, toggle_name, context):
        serialized_context = json.dumps(context)
        variant_json_ptr = self.lib.engine_get_variant(
            self.state, toggle_name.encode("utf-8"), serialized_context.encode("utf-8")
        )
        if variant_json_ptr:
            variant_json = variant_json_ptr.decode("utf-8")
            return json.loads(variant_json)
        else:
            return None
