@echo off
setlocal EnableExtensions
set "HOST=mac"
set "DEST=C:\ai\anzoth-cli\releases\macos-arm64-fast\anzoth"

ssh %HOST% "set -e; cd ~/anzoth-mac-validation; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; rustup target add aarch64-apple-darwin; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target aarch64-apple-darwin; ./codex-rs/target/aarch64-apple-darwin/fast-release/anzoth --version; ls -lh codex-rs/target/aarch64-apple-darwin/fast-release/anzoth; shasum -a 256 codex-rs/target/aarch64-apple-darwin/fast-release/anzoth; file codex-rs/target/aarch64-apple-darwin/fast-release/anzoth; lipo -info codex-rs/target/aarch64-apple-darwin/fast-release/anzoth" || exit /b 1

if not exist "C:\ai\anzoth-cli\releases\macos-arm64-fast" mkdir "C:\ai\anzoth-cli\releases\macos-arm64-fast"
scp %HOST%:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/fast-release/anzoth "%DEST%" || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
