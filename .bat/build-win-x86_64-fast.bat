@echo off
setlocal EnableExtensions
set "REPO=C:\ai\anzoth-cli\anzoth-cli"
set "BIN=%REPO%\codex-rs\target\fast-release\anzoth.exe"
set "DEST=C:\ai\anzoth-cli\releases\windows-x86_64-fast\anzoth.exe"

cd /d "%REPO%" || exit /b 1
echo FAST DEVELOPMENT BUILD
git fetch origin || exit /b 1
git checkout anzoth-rebrand || exit /b 1
git pull --ff-only origin anzoth-rebrand || exit /b 1
git rev-parse HEAD || exit /b 1

cargo build --profile fast-release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth || exit /b 1
"%BIN%" --version || exit /b 1

if not exist "C:\ai\anzoth-cli\releases\windows-x86_64-fast" mkdir "C:\ai\anzoth-cli\releases\windows-x86_64-fast"
copy /y "%BIN%" "%DEST%" >nul || exit /b 1
"%DEST%" --version || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
