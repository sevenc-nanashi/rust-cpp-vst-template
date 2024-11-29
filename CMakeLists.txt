cmake_minimum_required(VERSION 3.24)

add_subdirectory(deps/corrosion)
add_subdirectory(deps/dpf)

set(NAME "my_plugin")

project(${NAME})

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
  add_definitions(-DDEBUG)
  set(PLUGIN_NAME "${NAME}_debug")
  set(RUST_PROFILE "dev")
elseif(CMAKE_BUILD_TYPE STREQUAL "Release")
  set(PLUGIN_NAME "${NAME}")
  set(RUST_PROFILE "release")
elseif(NOT CMAKE_BUILD_TYPE)
  message(FATAL_ERROR "Build type not set")
else()
  message(FATAL_ERROR "Unknown build type: ${CMAKE_BUILD_TYPE}")
endif()

corrosion_import_crate(
  MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml
  PROFILE ${RUST_PROFILE}
  FEATURES static_link_windows
  CRATES ${NAME})

# Windows：utf-8を使う
if(MSVC)
  add_compile_options("/utf-8")
endif()

dpf_add_plugin(
  ${PLUGIN_NAME}
  TARGETS vst3
  UI_TYPE external
  FILES_DSP src/plugin.cpp
  FILES_UI src/ui.cpp)

target_link_libraries(${PLUGIN_NAME} PRIVATE ${NAME})
target_include_directories(${PLUGIN_NAME} PUBLIC "src" "deps")
target_compile_features(${PLUGIN_NAME} PUBLIC cxx_std_20)
