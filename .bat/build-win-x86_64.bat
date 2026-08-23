@echo off
setlocal EnableExtensions
set "REPO=C:\ai\anzoth-cli\anzoth-cli"
set "BIN=%REPO%\codex-rs\target\release\anzoth.exe"
set "DEST=C:\ai\anzoth-cli\releases\windows-x86_64\anzoth.exe"

cd /d "%REPO%" || exit /b 1
git fetch origin || exit /b 1
git checkout anzoth-rebrand || exit /b 1
git pull --ff-only origin anzoth-rebrand || exit /b 1
git rev-parse HEAD || exit /b 1

cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth || exit /b 1
"%BIN%" --version || exit /b 1

for /f "usebackq delims=" %%I in (`powershell -NoProfile -Command "$p=Get-ChildItem -Path $env:USERPROFILE'\.rustup\toolchains' -Filter llvm-strip.exe -Recurse -File -ErrorAction SilentlyContinue ^| Where-Object {$_.FullName -match 'x86_64-pc-windows-msvc\\bin\\llvm-strip\.exe$'} ^| Sort-Object FullName -Descending ^| Select-Object -First 1 -ExpandProperty FullName; if(-not $p){exit 1}; $p"`) do set "STRIP=%%I"
if not defined STRIP (
  echo ERROR: llvm-strip.exe not found.
  exit /b 1
)

"%STRIP%" "%BIN%" || exit /b 1
"%BIN%" --version || exit /b 1
if not exist "C:\ai\anzoth-cli\releases\windows-x86_64" mkdir "C:\ai\anzoth-cli\releases\windows-x86_64"
copy /y "%BIN%" "%DEST%" >nul || exit /b 1
"%DEST%" --version || exit /b 1
powershell -NoProfile -Command "$f=Get-Item -LiteralPath '%DEST%';Write-Host ('Bytes: '+$f.Length);Write-Host ('SHA256: '+(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)"
