cmake_minimum_required(VERSION 3.10)
project(mnn_llm_wrapper VERSION 1.0.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(MSVC)
    add_compile_options(/W3 /utf-8)
    add_compile_definitions(_CRT_SECURE_NO_WARNINGS)
endif()

set(MNN_DIR "" CACHE PATH "Path to MNN source directory")
set(MNN_BUILD_DIR "" CACHE PATH "Path to MNN build directory")

if(NOT MNN_DIR)
    get_filename_component(MNN_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../MNN" ABSOLUTE)
endif()

if(NOT MNN_BUILD_DIR)
    set(MNN_BUILD_DIR "${MNN_DIR}/build_llm")
endif()

set(MNN_INCLUDE_DIRS
    "${MNN_DIR}/include"
    "${MNN_DIR}/source"
    "${MNN_DIR}/express"
    "${MNN_DIR}/transformers/llm/engine/include"
)

set(MNN_LIBRARY_DIRS
    "${MNN_BUILD_DIR}/Release"
    "${MNN_BUILD_DIR}"
)

find_path(MNN_H_FOUND MNN/Interpreter.hpp PATHS ${MNN_INCLUDE_DIRS})
if(NOT MNN_H_FOUND)
    message(FATAL_ERROR "MNN headers not found. Please set MNN_DIR correctly.")
endif()

find_library(MNN_LIB_FOUND NAMES MNN libMNN PATHS ${MNN_LIBRARY_DIRS})
if(NOT MNN_LIB_FOUND)
    message(FATAL_ERROR "MNN library not found. Please set MNN_BUILD_DIR correctly.")
endif()

message(STATUS "MNN include dirs: ${MNN_INCLUDE_DIRS}")
message(STATUS "MNN library dirs: ${MNN_LIBRARY_DIRS}")
message(STATUS "MNN library: ${MNN_LIB_FOUND}")

add_library(mnn_llm_wrapper SHARED
    mnn_c_api.cpp
)

target_include_directories(mnn_llm_wrapper PRIVATE
    ${MNN_INCLUDE_DIRS}
    ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_directories(mnn_llm_wrapper PRIVATE
    ${MNN_LIBRARY_DIRS}
)

target_link_libraries(mnn_llm_wrapper PRIVATE MNN)

target_compile_definitions(mnn_llm_wrapper PRIVATE 
    MNN_BUILD_LLM 
    MNN_WRAPPER_EXPORTS
)

set_target_properties(mnn_llm_wrapper PROPERTIES
    OUTPUT_NAME "mnn_llm_wrapper"
    VERSION ${PROJECT_VERSION}
    SOVERSION 1
    PUBLIC_HEADER "mnn_c_api.h"
)

if(WIN32)
    install(TARGETS mnn_llm_wrapper
        RUNTIME DESTINATION bin
        LIBRARY DESTINATION lib
        ARCHIVE DESTINATION lib
    )
else()
    install(TARGETS mnn_llm_wrapper
        LIBRARY DESTINATION lib
        ARCHIVE DESTINATION lib
    )
endif()

install(FILES mnn_c_api.h DESTINATION include)
