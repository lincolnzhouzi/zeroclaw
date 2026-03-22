@echo off
setlocal enabledelayedexpansion

echo ========================================
echo MNN Build Script for Windows
echo ========================================

set CMAKE_PATH=D:\cmake-4.3.0-windows-x86_64\bin
set MNN_SOURCE=%~dp0..\..\..\..\MNN
set BUILD_DIR=%~dp0build
set WRAPPER_DIR=%~dp0

if not exist "%CMAKE_PATH%\cmake.exe" (
    echo ERROR: CMake not found at %CMAKE_PATH%
    echo Please install CMake or update the CMAKE_PATH variable
    exit /b 1
)

if not exist "%MNN_SOURCE%\CMakeLists.txt" (
    echo ERROR: MNN source not found at %MNN_SOURCE%
    echo Please clone MNN repository or set MNN_SOURCE variable
    exit /b 1
)

echo Checking for available compilers...

set GENERATOR=
set USE_NINJA=0

where ninja >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo Found Ninja build system
    set GENERATOR=Ninja
    set USE_NINJA=1
)

if "%GENERATOR%"=="" (
    if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" (
        echo Found Visual Studio 2022 Community
        set GENERATOR=Visual Studio 17 2022
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Professional\Common7\Tools\VsDevCmd.bat" (
        echo Found Visual Studio 2022 Professional
        set GENERATOR=Visual Studio 17 2022
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\Common7\Tools\VsDevCmd.bat" (
        echo Found Visual Studio 2022 Enterprise
        set GENERATOR=Visual Studio 17 2022
    ) else if exist "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\Common7\Tools\VsDevCmd.bat" (
        echo Found Visual Studio 2019 Community
        set GENERATOR=Visual Studio 16 2019
    ) else if exist "C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional\Common7\Tools\VsDevCmd.bat" (
        echo Found Visual Studio 2019 Professional
        set GENERATOR=Visual Studio 16 2019
    ) else if exist "C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\Common7\Tools\VsDevCmd.bat" (
        echo Found Visual Studio 2019 Enterprise
        set GENERATOR=Visual Studio 16 2019
    )
)

if "%GENERATOR%"=="" (
    echo.
    echo ========================================
    echo ERROR: No suitable compiler found!
    echo ========================================
    echo.
    echo Please install one of the following:
    echo   - Visual Studio 2019 or 2022
    echo   - MinGW-w64 with Ninja
    echo.
    echo Or download pre-built MNN libraries:
    echo   powershell -ExecutionPolicy Bypass -File download_mnn.ps1
    echo.
    exit /b 1
)

echo Using generator: %GENERATOR%
echo.

echo [1/3] Configuring MNN build...

if not exist "%BUILD_DIR%" mkdir "%BUILD_DIR%"
cd /d "%BUILD_DIR%"

set CMAKE_ARGS=-DCMAKE_BUILD_TYPE=Release ^
    -DMNN_BUILD_SHARED_LIBS=ON ^
    -DMNN_SEP_BUILD=OFF ^
    -DMNN_BUILD_TRAIN=OFF ^
    -DMNN_BUILD_DEMO=OFF ^
    -DMNN_BUILD_TOOLS=OFF ^
    -DMNN_BUILD_QUANTOOLS=OFF ^
    -DMNN_EVALUATION=OFF ^
    -DMNN_BUILD_CONVERTER=OFF ^
    -DMNN_REDUCE_SIZE=OFF ^
    -DMNN_DEBUG_MEMORY=OFF ^
    -DMNN_DEBUG_TENSOR_SIZE=OFF ^
    -DMNN_GPU_TRACE=OFF ^
    -DMNN_BUILD_LLM=ON ^
    -DMNN_BUILD_LLM_OMNI=OFF ^
    -DMNN_BUILD_DIFFUSION=OFF ^
    -DMNN_JNI=OFF ^
    -DMNN_LOW_MEMORY=ON ^
    -DMNN_USE_SSE=ON ^
    -DMNN_SUPPORT_QUANT_EXTEND=ON

if "%GENERATOR%"=="Ninja" (
    "%CMAKE_PATH%\cmake.exe" "%MNN_SOURCE%" -G "Ninja" %CMAKE_ARGS%
) else (
    "%CMAKE_PATH%\cmake.exe" "%MNN_SOURCE%" -G "%GENERATOR%" -A x64 %CMAKE_ARGS%
)

if %ERRORLEVEL% neq 0 (
    echo ERROR: MNN CMake configuration failed
    exit /b 1
)

echo [2/3] Building MNN...

if "%USE_NINJA%"=="1" (
    "%CMAKE_PATH%\cmake.exe" --build . --config Release
) else (
    "%CMAKE_PATH%\cmake.exe" --build . --config Release --parallel 8
)

if %ERRORLEVEL% neq 0 (
    echo ERROR: MNN build failed
    exit /b 1
)

echo [3/3] Building MNN Wrapper...

set WRAPPER_BUILD_DIR=%WRAPPER_DIR%build
if not exist "%WRAPPER_BUILD_DIR%" mkdir "%WRAPPER_BUILD_DIR%"
cd /d "%WRAPPER_BUILD_DIR%"

set WRAPPER_ARGS=-DCMAKE_BUILD_TYPE=Release ^
    -DMNN_SOURCE_DIR="%MNN_SOURCE%" ^
    -DMNN_BUILD_DIR="%BUILD_DIR%\Release" ^
    -DMNN_BUILD_LLM=ON ^
    -DMNN_BUILD_SHARED=ON

if "%GENERATOR%"=="Ninja" (
    "%CMAKE_PATH%\cmake.exe" "%WRAPPER_DIR%" -G "Ninja" %WRAPPER_ARGS%
) else (
    "%CMAKE_PATH%\cmake.exe" "%WRAPPER_DIR%" -G "%GENERATOR%" -A x64 %WRAPPER_ARGS%
)

if %ERRORLEVEL% neq 0 (
    echo ERROR: Wrapper CMake configuration failed
    exit /b 1
)

if "%USE_NINJA%"=="1" (
    "%CMAKE_PATH%\cmake.exe" --build . --config Release
) else (
    "%CMAKE_PATH%\cmake.exe" --build . --config Release --parallel 4
)

if %ERRORLEVEL% neq 0 (
    echo ERROR: Wrapper build failed
    exit /b 1
)

echo ========================================
echo Build completed successfully!
echo ========================================
echo.
echo MNN Library: %BUILD_DIR%\Release\MNN.dll
echo Wrapper Library: %WRAPPER_BUILD_DIR%\Release\mnn_wrapper.dll
echo.
echo To use the library, set environment variable:
echo   set MNN_LIB_DIR=%BUILD_DIR%\Release
echo.

cd /d "%~dp0"
endlocal
