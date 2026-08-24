@echo off
setlocal EnableExtensions
set "REPO=C:\ai\anzoth-cli\anzoth-cli"
set "BIN=%REPO%\codex-rs\target\release\anzoth.exe"
set "DEST=C:\ai\anzoth-cli\releases\windows-x86_64-debug\anzoth.exe"

cd /d "%REPO%" || exit /b 1
echo DEBUG-SYMBOL RELEASE BUILD
git fetch origin || exit /b 1
git checkout anzoth-rebrand || exit /b 1
git pull --ff-only origin anzoth-rebrand || exit /b 1
git rev-parse HEAD || exit /b 1

cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth || exit /b 1
"%BIN%" --version || exit /b 1
if not exist "C:\ai\anzoth-cli\releases\windows-x86_64-debug" mkdir "C:\ai\anzoth-cli\releases\windows-x86_64-debug"
copy /y "%BIN%" "%DEST%" >nul || exit /b 1
"%DEST%" --version || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
