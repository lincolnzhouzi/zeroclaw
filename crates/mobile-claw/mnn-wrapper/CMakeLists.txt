cmake_minimum_required(VERSION 3.15)
project(mnn_wrapper C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

option(MNN_BUILD_LLM "Build with LLM support" ON)
option(MNN_BUILD_SHARED "Build shared library" ON)

set(MNN_SOURCE_DIR "" CACHE PATH "Path to MNN source directory")
set(MNN_BUILD_DIR "" CACHE PATH "Path to MNN build directory")

if(NOT MNN_SOURCE_DIR)
    message(FATAL_ERROR "MNN_SOURCE_DIR must be set to MNN source directory")
endif()

if(MNN_BUILD_SHARED)
    add_library(mnn_wrapper SHARED
        mnn_c_api.cpp
    )
else()
    add_library(mnn_wrapper STATIC
        mnn_c_api.cpp
    )
endif()

target_include_directories(mnn_wrapper PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${MNN_SOURCE_DIR}/include
    ${MNN_SOURCE_DIR}/source
    ${MNN_SOURCE_DIR}/express
    ${MNN_SOURCE_DIR}/transformers/llm/engine/include
)

if(MNN_BUILD_DIR)
    target_link_directories(mnn_wrapper PRIVATE ${MNN_BUILD_DIR})
endif()

target_link_libraries(mnn_wrapper PRIVATE
    MNN
)

if(MNN_BUILD_LLM)
    target_compile_definitions(mnn_wrapper PRIVATE MNN_BUILD_LLM)
    target_link_libraries(mnn_wrapper PRIVATE llm)
endif()

if(WIN32)
    target_compile_definitions(mnn_wrapper PRIVATE 
        _CRT_SECURE_NO_WARNINGS
        NOMINMAX
    )
    if(MSVC)
        target_compile_options(mnn_wrapper PRIVATE /utf-8)
    endif()
endif()

set_target_properties(mnn_wrapper PROPERTIES
    OUTPUT_NAME mnn_wrapper
    VERSION 1.0.0
    SOVERSION 1
)

install(TARGETS mnn_wrapper
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
)

install(FILES mnn_c_api.h
    DESTINATION include
)
