@echo off
setlocal EnableExtensions
set "HOST=anzoth-dev"
set "DEST=C:\ai\anzoth-cli\releases\linux-x86_64\anzoth"

ssh %HOST% "set -e; cd ~/anzoth-linux-validation; echo PRODUCTION RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; export CARGO_PROFILE_RELEASE_DEBUG=0; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; mkdir -p dist/linux-x64; cp codex-rs/target/release/anzoth dist/linux-x64/anzoth; strip dist/linux-x64/anzoth; ./dist/linux-x64/anzoth --version; ls -lh dist/linux-x64/anzoth; sha256sum dist/linux-x64/anzoth; file dist/linux-x64/anzoth" || exit /b 1


if not exist "C:\ai\anzoth-cli\releases\linux-x86_64" mkdir "C:\ai\anzoth-cli\releases\linux-x86_64"
scp %HOST%:~/anzoth-linux-validation/dist/linux-x64/anzoth "%DEST%" || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
