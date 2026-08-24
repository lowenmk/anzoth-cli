@echo off
setlocal EnableExtensions
set "HOST=mac"
set "DEST=C:\ai\anzoth-cli\releases\macos-x86_64\anzoth"

ssh %HOST% "set -e; cd ~/anzoth-mac-validation; echo PRODUCTION RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; export CARGO_PROFILE_RELEASE_DEBUG=0; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target x86_64-apple-darwin; strip codex-rs/target/x86_64-apple-darwin/release/anzoth; ./codex-rs/target/x86_64-apple-darwin/release/anzoth --version; ls -lh codex-rs/target/x86_64-apple-darwin/release/anzoth; shasum -a 256 codex-rs/target/x86_64-apple-darwin/release/anzoth; file codex-rs/target/x86_64-apple-darwin/release/anzoth" || exit /b 1


if not exist "C:\ai\anzoth-cli\releases\macos-x86_64" mkdir "C:\ai\anzoth-cli\releases\macos-x86_64"
scp %HOST%:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/release/anzoth "%DEST%" || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
