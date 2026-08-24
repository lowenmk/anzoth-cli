@echo off
setlocal EnableExtensions
set "HOST=mac"
set "DEST=C:\ai\anzoth-cli\releases\macos-arm64-debug\anzoth"

ssh %HOST% "set -e; cd ~/anzoth-mac-validation; echo DEBUG-SYMBOL RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; rustup target add aarch64-apple-darwin; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target aarch64-apple-darwin; ls -lh codex-rs/target/aarch64-apple-darwin/release/anzoth; shasum -a 256 codex-rs/target/aarch64-apple-darwin/release/anzoth; file codex-rs/target/aarch64-apple-darwin/release/anzoth; lipo -info codex-rs/target/aarch64-apple-darwin/release/anzoth" || exit /b 1

if not exist "C:\ai\anzoth-cli\releases\macos-arm64-debug" mkdir "C:\ai\anzoth-cli\releases\macos-arm64-debug"
scp %HOST%:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/release/anzoth "%DEST%" || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
