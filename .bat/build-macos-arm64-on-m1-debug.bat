@echo off
setlocal EnableExtensions
set "HOST=mac-m1"
set "REMOTE_REPO=~/anzoth-mac-validation"
set "DEST=C:\ai\anzoth-cli\releases\macos-arm64-debug\anzoth"

if not "%~1"=="" set "REMOTE_REPO=%~1"

ssh %HOST% "set -e; test -d %REMOTE_REPO%/.git || { echo ERROR: repo not found at %REMOTE_REPO%; exit 2; }; cd %REMOTE_REPO%; echo DEBUG-SYMBOL RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; ./codex-rs/target/release/anzoth --version; ls -lh codex-rs/target/release/anzoth; shasum -a 256 codex-rs/target/release/anzoth; file codex-rs/target/release/anzoth; lipo -info codex-rs/target/release/anzoth" || exit /b 1

if not exist "C:\ai\anzoth-cli\releases\macos-arm64-debug" mkdir "C:\ai\anzoth-cli\releases\macos-arm64-debug"
scp %HOST%:%REMOTE_REPO%/codex-rs/target/release/anzoth "%DEST%" || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
