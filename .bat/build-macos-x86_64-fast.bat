@echo off
setlocal EnableExtensions
set "SCRIPT_NAME=build-macos-x86_64-fast"
set "PS_EXE=pwsh.exe"
where pwsh.exe >nul 2>nul || set "PS_EXE=powershell.exe"
start "" "%PS_EXE%" -NoLogo -NoExit -ExecutionPolicy Bypass -File "%~dp0build-launch.ps1" -ScriptName "%SCRIPT_NAME%"
