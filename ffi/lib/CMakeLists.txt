cmake_minimum_required(VERSION 3.20)

project(cpp)

set(CMAKE_CXX_VERSION 11)

add_executable(main src/main.cpp)
add_custom_target(rust_lib_build
    COMMAND cargo build
)
add_library(rust_lib INTERFACE)
target_include_directories(rust_lib INTERFACE target)
target_link_libraries(rust_lib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/target/debug/libffi_lib.a)
add_dependencies(rust_lib rust_lib_build)
target_link_libraries(main rust_lib)
