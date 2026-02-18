cmake_minimum_required(VERSION 3.22)
project(decision_engine LANGUAGES CXX)

# Prefer target-based standard, but keep this if you want global
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Optional: static libs
option(BUILD_SHARED_LIBS "Build shared libraries" OFF)

if(NOT DEFINED CMAKE_TOOLCHAIN_FILE)
    set(CMAKE_TOOLCHAIN_FILE
        "${CMAKE_SOURCE_DIR}/vcpkg/scripts/buildsystems/vcpkg.cmake"
        CACHE STRING "Vcpkg toolchain file"
    )
endif()

# Dependencies
find_package(nlohmann_json CONFIG REQUIRED)
find_package(spdlog CONFIG REQUIRED)
find_package(dlib CONFIG REQUIRED)

# Library
add_library(recommendation_engine STATIC
    src/recommendation_engine.cpp
)

target_compile_features(recommendation_engine PUBLIC cxx_std_20)

target_include_directories(recommendation_engine PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

target_link_libraries(recommendation_engine PUBLIC
    nlohmann_json::nlohmann_json
    spdlog::spdlog
    dlib::dlib
)
