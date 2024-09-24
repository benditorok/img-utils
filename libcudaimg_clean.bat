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

:: Clean the solution
echo Cleaning the solution...
msbuild "libcudaimg.sln" /t:Clean /p:Configuration=%CONFIG%

:: Check if the clean was successful
if %errorlevel% neq 0 (
    echo Clean failed!
    exit /b %errorlevel%
)

echo Clean completed successfully!
endlocal