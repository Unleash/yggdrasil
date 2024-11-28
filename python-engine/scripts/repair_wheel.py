import os
import glob
import sysconfig
import zipfile

python_version = f"cp{sysconfig.get_python_version().replace('.', '')}"
abi_tag = "abi3"
platform_tag = os.getenv("PLATFORM_TAG", "manylinux2014_x86_64")
wheels = glob.glob("dist/*-py3-none-any.whl")

for whl in wheels:
    base_name = os.path.basename(whl)[:-4]

    new_name = f"{base_name.replace('py3-none-any', f'{python_version}-{abi_tag}-{platform_tag}')}.whl"

    old_path = os.path.join("dist", os.path.basename(whl))
    new_path = os.path.join("staging", new_name)

    with zipfile.ZipFile(old_path) as old_zip_file:
        with zipfile.ZipFile(new_path, 'a') as new_zip_file:
            for item in old_zip_file.namelist():

                if item.endswith(".dist-info/WHEEL"):
                    wheel_data = old_zip_file.read(item).decode().splitlines()
                    new_wheel_data = []
                    for line in wheel_data:
                        if line.startswith("Tag:"):
                            new_wheel_data.append(f"Tag: py3-{abi_tag}-{platform_tag}")
                        elif line.startswith("Root-Is-Purelib:"):
                            new_wheel_data.append("Root-Is-Purelib: false")
                        else:
                            new_wheel_data.append(line)
                    new_zip_file.writestr(item, "\n".join(new_wheel_data))
                else:
                    new_zip_file.writestr(item, old_zip_file.read(item))

    print(f"Repaired wheel at {old_path} and moved to {new_path}")