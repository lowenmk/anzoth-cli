@echo off
setlocal EnableExtensions
set "HOST=mac-m1"
set "REMOTE_REPO=~/anzoth-mac-validation"
set "DEST=C:\ai\anzoth-cli\releases\macos-arm64-fast\anzoth"

if not "%~1"=="" set "REMOTE_REPO=%~1"

ssh %HOST% "set -e; test -d %REMOTE_REPO%/.git || { echo ERROR: repo not found at %REMOTE_REPO%; exit 2; }; cd %REMOTE_REPO%; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; ./codex-rs/target/fast-release/anzoth --version; ls -lh codex-rs/target/fast-release/anzoth; shasum -a 256 codex-rs/target/fast-release/anzoth; file codex-rs/target/fast-release/anzoth; lipo -info codex-rs/target/fast-release/anzoth" || exit /b 1

if not exist "C:\ai\anzoth-cli\releases\macos-arm64-fast" mkdir "C:\ai\anzoth-cli\releases\macos-arm64-fast"
scp %HOST%:%REMOTE_REPO%/codex-rs/target/fast-release/anzoth "%DEST%" || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
