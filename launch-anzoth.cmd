@echo off
setlocal
set "SCRIPT_DIR=%~dp0"
set "EXECUTABLE=%SCRIPT_DIR%codex-rs\target\debug\anzoth.exe"

if not exist "%EXECUTABLE%" (
    echo Missing executable: "%EXECUTABLE%"
    exit /b 1
)

pushd "%SCRIPT_DIR%"
"%EXECUTABLE%" %*
set "EXIT_CODE=%ERRORLEVEL%"
popd
exit /b %EXIT_CODE%
