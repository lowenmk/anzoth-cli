@echo off
setlocal EnableExtensions
set "SCRIPT_NAME=build-macos-arm64-on-m1-fast"
set "PS_EXE=pwsh.exe"
where pwsh.exe >nul 2>nul || set "PS_EXE=powershell.exe"
"%PS_EXE%" -NoLogo -NoExit -ExecutionPolicy Bypass -File "%~dp0build-launch.ps1" -ScriptName "%SCRIPT_NAME%"