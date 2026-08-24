@echo off
setlocal EnableExtensions
set "HOST=anzoth-dev"
set "DEST=C:\ai\anzoth-cli\releases\linux-x86_64-fast\anzoth"

ssh %HOST% "set -e; cd ~/anzoth-linux-validation; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; mkdir -p dist/linux-x64-fast; cp codex-rs/target/fast-release/anzoth dist/linux-x64-fast/anzoth; ./dist/linux-x64-fast/anzoth --version; ls -lh dist/linux-x64-fast/anzoth; sha256sum dist/linux-x64-fast/anzoth; file dist/linux-x64-fast/anzoth" || exit /b 1

if not exist "C:\ai\anzoth-cli\releases\linux-x86_64-fast" mkdir "C:\ai\anzoth-cli\releases\linux-x86_64-fast"
scp %HOST%:~/anzoth-linux-validation/dist/linux-x64-fast/anzoth "%DEST%" || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
