import os
import glob
import sysconfig

python_version = f"cp{sysconfig.get_python_version().replace('.', '')}"
abi_tag = python_version

platform_tag = os.getenv("PLATFORM_TAG", "manylinux2014_x86_64")

wheels = glob.glob("dist/*-py3-none-any.whl")

for whl in wheels:
    base_name = os.path.basename(whl)[:-4]

    new_name = f"{base_name.replace('py3-none-any', f'{python_version}-{abi_tag}-{platform_tag}')}.whl"

    old_path = os.path.join("dist", os.path.basename(whl))
    new_path = os.path.join("staging", new_name)

    os.rename(old_path, new_path)
    print(f"Renamed {old_path} to {new_path}")