@echo off
setlocal

:: Set the build configuration
set ARCH=x64
set CONFIG=Release

:: Call vcvars64.bat to set up the environment
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

:: Add msbuild to the PATH
set "MSBUILD_PATH=C:\Program Files\Microsoft Visual Studio\2022\Community\MSBuild\Current\Bin"
set "PATH=%MSBUILD_PATH%;%PATH%"

:: Navigate to the project directory
cd /d "%~dp0\libcudaimg"

:: Check if the solution file exists
if not exist "libcudaimg.sln" (
    echo Solution file libcudaimg.sln not found!
    exit /b 1
)

:: Build the CUDA library
echo Building CUDA library in %CONFIG% mode...
msbuild "libcudaimg.sln" /p:Configuration=%CONFIG%

:: Check if the build was successful
if %errorlevel% neq 0 (
    echo Build failed!
    exit /b %errorlevel%
)

:: Copy the DLL to the data folder
echo Copying libcudaimg.dll to data folder...
if not exist "%~dp0\data" (
    mkdir "%~dp0\data"
)
copy /y "%~dp0\libcudaimg\%ARCH%\%CONFIG%\libcudaimg.dll" "%~dp0\data\"

:: Check if the copy was successful
if %errorlevel% neq 0 (
    echo Copy failed!
    exit /b %errorlevel%
)

echo Build and copy completed successfully!
endlocal