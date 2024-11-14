import os
from setuptools import Distribution, Extension

def build(target_platform="linux"):
    lib_dir = os.path.join(os.path.dirname(__file__), "lib")

    if target_platform == "linux":
        shared_lib_name = "libyggdrasilffi.so"
    elif target_platform == "macos":
        shared_lib_name = "libyggdrasilffi.dylib"
    elif target_platform == "win32":
        shared_lib_name = "yggdrasilffi.dll"
    else:
        raise RuntimeError(f"Unsupported target platform: {target_platform}")

    ext_modules = [
        Extension(
            "yggdrasil_engine._native",
            sources=[],
            libraries=[shared_lib_name],
            library_dirs=[lib_dir],
        )
    ]

    distribution = Distribution({"name": "extended", "ext_modules": ext_modules})
    distribution.package_dir = "lib"

if __name__ == "__main__":
    # Read target platform from environment or default to 'linux'
    build(target_platform=os.environ.get("TARGET_PLATFORM", "linux"))