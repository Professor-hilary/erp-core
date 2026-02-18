# Decision Engine – Recommendation Library

## Overview

**Decision Engine** is a lightweight, high-performance **C++ recommendation engine** designed to provide intelligent suggestions based on input data.
It is intended to be a **modular library** that can be integrated into larger applications (e.g., ERP systems, e-commerce platforms, or analytics tools) to generate recommendations for products, services, or actions.

The engine is designed with **C++20**, emphasizing modern coding standards, static linking for stability, and easy integration with external libraries via **vcpkg**.

**Key intentions:**

- Serve as a reusable recommendation module in larger projects.
- Provide a clean interface for adding machine learning, heuristics, or rules-based recommendation logic.
- Maintain a minimal dependency footprint while leveraging high-quality C++ libraries like **dlib**, **spdlog**, and **nlohmann_json**.

---

## Project Structure

```

decision-engine/
│
├─ CMakeLists.txt            # Project build file
├─ vcpkg/                    # vcpkg repo (toolchain scripts)
├─ vcpkg_installed/          # Installed libraries (persistent)
├─ src/                      # Source files (library implementation)
│   └─ recommendation_engine.cpp
├─ include/                  # Header files
├─ build/                    # Build artifacts (safe to delete)
└─ tests/                    # Optional test executables

````

---

## Dependencies

All dependencies are managed via **vcpkg**:

- [dlib](http://dlib.net/) – for ML and numerical operations
- [nlohmann_json](https://github.com/nlohmann/json) – JSON parsing and serialization
- [spdlog](https://github.com/gabime/spdlog) – Logging

---

## Setup Instructions

### 1. Clone the project

```bash
git clone <your-repo-url>
cd decision-engine
````

### 2. Install vcpkg (if missing)

```bash
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
./bootstrap-vcpkg.sh
cd ..
```

### 3. Install dependencies

```bash
./vcpkg/vcpkg install nlohmann-json:x64-linux spdlog:x64-linux dlib:x64-linux
```

> **Note:** Do not delete the `vcpkg/scripts/buildsystems/vcpkg.cmake` file; it is required for CMake to find your dependencies.

---

## Build Instructions

1. Create a clean build directory:

```bash
rm -rf build
mkdir build
cd build
```

2. Configure the project with the vcpkg toolchain:

```bash
cmake .. \
  -DCMAKE_TOOLCHAIN_FILE=../vcpkg/scripts/buildsystems/vcpkg.cmake \
  -DVCPKG_TARGET_TRIPLET=x64-linux
```

3. Build the library:

```bash
cmake --build .
```

4. (Optional) Run a test executable if added:

```bash
./test_engine
```

---

## Using the Library

* The library is **static** by default.
* Include the headers in your project:

```cpp
#include "recommendation_engine.hpp"
```

* Link against `recommendation_engine` in your CMake project:

```cmake
target_link_libraries(your_project PRIVATE recommendation_engine)
target_include_directories(your_project PRIVATE path/to/include)
```

---

## Recommended Workflow

1. Edit source or include files
2. Commit changes often:

```bash
git add .
git commit -m "WIP: implement new feature"
```

3. Use a **clean build folder** for CMake to avoid stale cache issues
4. Run CMake with **vcpkg toolchain**
5. Build and test
6. Push commits to remote regularly

---

## Gotchas & Tips

| Problem                                       | Cause                      | Solution                                                                  |
| --------------------------------------------- | -------------------------- | ------------------------------------------------------------------------- |
| `find_package(dlib)` fails                    | Toolchain file missing     | Pass `-DCMAKE_TOOLCHAIN_FILE=...` when running CMake                      |
| Headers exist but CMake cannot find libraries | Missing `dlibConfig.cmake` | Restore vcpkg repo/toolchain; do not delete `vcpkg/scripts/buildsystems/` |
| `rm -rf *` deletes project                    | Dangerous command in root  | Only delete `build/`; alias `rm='rm -i'` for safety                       |
| Mixed system + vcpkg libs                     | ABI mismatch               | Use vcpkg consistently with the toolchain                                 |
| Build fails after modifying toolchain         | Stale cache                | Always delete `build/` before reconfiguring CMake                         |

---

## Future Directions

* Integrate **ML models** via dlib or other frameworks
* Add **rules-based recommendation system** support
* Expand **test coverage** and add benchmarking
* Optionally build a **shared library** for dynamic linking

---

## Resources

* [CMake Documentation](https://cmake.org/documentation/)
* [vcpkg Docs](https://github.com/microsoft/vcpkg)
* [dlib Tutorials](http://dlib.net/)
* [nlohmann_json Docs](https://nlohmann.github.io/json/)
* [spdlog Docs](https://github.com/gabime/spdlog)

---

> ⚡ **Pro Tip:** Always keep your source and include files under Git version control. The `build/` and `vcpkg_installed/` folders can be regenerated, so don’t commit them.

