# 🗂 Decision Engine Project Configuration Guide

## 1️⃣ Project Structure

Keep the project organized to prevent accidental deletion disasters:

```
decision-engine/
│
├─ CMakeLists.txt            # Project build file
├─ vcpkg/                    # vcpkg repo (toolchain scripts)
├─ vcpkg_installed/          # Installed libraries (vcpkg)
├─ src/                      # Source code
│   └─ recommendation_engine.cpp
├─ include/                  # Header files
├─ build/                    # Build artifacts (safe to delete)
└─ tests/                    # Optional: test programs
```

**Gotchas:**

* Never put source code or include files inside `build/`.
* Keep `vcpkg_installed/` intact; deleting it forces reinstall of dependencies.

---

## 2️⃣ Install CMake and Compiler

* **CMake:** ≥ 3.22 (supports `target_compile_features`, modern `find_package`, toolchain files)
* **Compiler:** GCC ≥ 15 or Clang ≥ 16 recommended for C++20 features

**Gotchas:**

* Check `c++ --version` before building.
* Older compilers may fail with `cxx_std_20` targets.

---

## 3️⃣ Setup vcpkg

1. Clone vcpkg repo (if missing or broken):

```bash
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
./bootstrap-vcpkg.sh
```

2. Install dependencies:

```bash
../vcpkg/vcpkg install nlohmann-json:x64-linux spdlog:x64-linux dlib:x64-linux
```

3. Verify installed files:

```bash
ls ../vcpkg_installed/x64-linux/share/
# Should show nlohmann_json, spdlog, dlib
```

**Gotchas:**

* The toolchain file `vcpkg/scripts/buildsystems/vcpkg.cmake` is required for CMake to locate libraries.
* Keep `vcpkg_installed/` folder; don’t delete unless you’re reinstalling packages.

---

## 4️⃣ Configure CMakeLists.txt

**Key points:**

* Use **C++20**
* Prefer **target-based linking**
* Use **toolchain file for vcpkg**

Example:

```cmake
cmake_minimum_required(VERSION 3.22)
project(decision_engine LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
option(BUILD_SHARED_LIBS "Build shared libraries" OFF)

# Toolchain must be specified at CMake configure time
# DO NOT try to set it inside CMakeLists for toolchain use
# It works only via -DCMAKE_TOOLCHAIN_FILE
# This is why we pass it when running cmake from the build folder

# Dependencies
find_package(nlohmann_json CONFIG REQUIRED)
find_package(spdlog CONFIG REQUIRED)
find_package(dlib CONFIG REQUIRED)

# Library
add_library(recommendation_engine STATIC src/recommendation_engine.cpp)
target_compile_features(recommendation_engine PUBLIC cxx_std_20)
target_include_directories(recommendation_engine PUBLIC include)
target_link_libraries(recommendation_engine PUBLIC
    nlohmann_json::nlohmann_json
    spdlog::spdlog
    dlib::dlib
)
```

**Gotchas:**

* `CMAKE_TOOLCHAIN_FILE` must be passed **via command line** or CMake cache, not set inside the file.
* Mixing system-installed and vcpkg libraries can cause **ABI mismatches**.

---

## 5️⃣ Building the project

1. Create a clean build folder:

```bash
rm -rf build
mkdir build
cd build
```

2. Configure CMake **with vcpkg toolchain**:

```bash
cmake .. \
  -DCMAKE_TOOLCHAIN_FILE=../vcpkg/scripts/buildsystems/vcpkg.cmake \
  -DVCPKG_TARGET_TRIPLET=x64-linux
```

3. Build the project:

```bash
cmake --build .
```

4. Optional: run your executable (if you created one):

```bash
./test_engine
```

**Gotchas:**

* Always start from a **clean build folder** after toolchain changes.
* Never run `rm -rf *` in the project root — only safe to do inside `build/`.
* If CMake can’t find a library, ensure you installed it for the correct triplet (x64-linux).

---

## 6️⃣ Version control safety

* Commit **all source and include files** frequently:

```bash
git add .
git commit -m "WIP: implement feature X"
```

* Push to remote regularly:

```bash
git push origin main
```

* Keep `build/` and `vcpkg_installed/` **ignored**:

```gitignore
/build/
/vcpkg_installed/
```

* Use branches for experiments; never risk master.

**Gotchas:**

* Never rely on uncommitted changes as your “backup.”
* If you accidentally delete `.git` or toolchain files, recovery is painful.

---

## 7️⃣ Optional: Testing & Executables

* Add a test folder:

```
tests/main.cpp
```

Example test CMake snippet:

```cmake
add_executable(test_engine tests/main.cpp)
target_link_libraries(test_engine PRIVATE recommendation_engine)
```

* Run your tests via:

```bash
./test_engine
```

**Gotchas:**

* Test executable is **required** to validate the library’s functionality.
* Static libraries cannot run directly.

---

## 8️⃣ Recommended workflow

1. **Edit source or include files**
2. **Commit frequently**
3. **Use clean build folder**
4. **Run CMake with toolchain**
5. **Build**
6. **Test**

**Optional:** Push new changes after passing tests.

---

## 9️⃣ Recovery lessons from past incidents

* Keep **toolchain scripts and vcpkg repo** separate from installed packages.
* Never delete root project files.
* Version-control your **CMakeLists.txt, src/, include/** folder — not `build/`.
* Keep a habit of `git commit -m "WIP"` even for small changes.

---

### 🔹 TL;DR Gotchas

| Problem                                  | Cause                      | Solution                                             |
| ---------------------------------------- | -------------------------- | ---------------------------------------------------- |
| `find_package(dlib)` fails               | No toolchain file          | Pass `-DCMAKE_TOOLCHAIN_FILE=...` when running CMake |
| Headers exist but CMake cannot find libs | Missing `dlibConfig.cmake` | Restore vcpkg repo/toolchain, rebuild CMake          |
| rm -rf deletes project                   | Dangerous command in root  | Only delete `build/`; alias `rm='rm -i'`             |
| Mixed system + vcpkg libs                | ABI incompatibility        | Use **vcpkg manifest/toolchain consistently**        |
