param(
    [Parameter(Mandatory = $true)]
    [ValidateSet(
        'build-win-x86_64',
        'build-win-x86_64-fast',
        'build-win-x86_64-debug',
        'build-macos-x86_64',
        'build-macos-x86_64-fast',
        'build-macos-x86_64-debug',
        'build-macos-arm64-on-intel',
        'build-macos-arm64-on-intel-fast',
        'build-macos-arm64-on-intel-debug',
        'build-macos-arm64-on-m1',
        'build-macos-arm64-on-m1-fast',
        'build-macos-arm64-on-m1-debug',
        'build-linux-x86_64',
        'build-linux-x86_64-fast',
        'build-linux-x86_64-debug'
    )]
    [string]$ScriptName
)

$ErrorActionPreference = 'Stop'
$Repo = Split-Path -Parent $PSScriptRoot

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [scriptblock]$Script,
        [Parameter(Mandatory = $true)]
        [string]$Label
    )

    & $Script
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

function Ensure-Directory {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -ItemType Directory -Force -Path $Path | Out-Null
    }
}

function Print-FileStats {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $f = Get-Item -LiteralPath $Path
    Write-Host ('Bytes: ' + $f.Length)
    Write-Host ('SHA256: ' + (Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash)
}

function Refresh-Repo {
    Invoke-Checked { git fetch origin } 'git fetch origin'
    Invoke-Checked { git checkout anzoth-rebrand } 'git checkout anzoth-rebrand'
    Invoke-Checked { git pull --ff-only origin anzoth-rebrand } 'git pull --ff-only origin anzoth-rebrand'
    Invoke-Checked { git rev-parse HEAD } 'git rev-parse HEAD'
}

Set-Location -LiteralPath $Repo

switch ($ScriptName) {
    'build-win-x86_64' {
        $bin = Join-Path $Repo 'codex-rs\target\release\anzoth.exe'
        $sandbox = Join-Path $Repo 'codex-rs\target\release\codex-windows-sandbox-setup.exe'
        $dest = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth.exe'
        $resourcesDir = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth-resources'
        $sandboxDest = Join-Path $resourcesDir 'codex-windows-sandbox-setup.exe'
        Write-Host 'PRODUCTION RELEASE BUILD'
        Refresh-Repo
        Invoke-Checked { cargo build --release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth } 'cargo build'
        Invoke-Checked { cargo build --release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-windows-sandbox --bin codex-windows-sandbox-setup } 'cargo build sandbox helper'
        Invoke-Checked { & $bin --version } 'source binary version'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\windows-x86_64'
        Ensure-Directory $resourcesDir
        Copy-Item -LiteralPath $bin -Destination $dest -Force
        Copy-Item -LiteralPath $sandbox -Destination $sandboxDest -Force
        Invoke-Checked { & $dest --version } 'destination binary version'
        Print-FileStats $dest
        break
    }

    'build-win-x86_64-fast' {
        $bin = Join-Path $Repo 'codex-rs\target\fast-release\anzoth.exe'
        $sandbox = Join-Path $Repo 'codex-rs\target\fast-release\codex-windows-sandbox-setup.exe'
        $dest = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth.exe'
        $resourcesDir = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth-resources'
        $sandboxDest = Join-Path $resourcesDir 'codex-windows-sandbox-setup.exe'
        Write-Host 'FAST DEVELOPMENT BUILD'
        Refresh-Repo
        Invoke-Checked { cargo build --profile fast-release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth } 'cargo build'
        Invoke-Checked { cargo build --profile fast-release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-windows-sandbox --bin codex-windows-sandbox-setup } 'cargo build sandbox helper'
        Invoke-Checked { & $bin --version } 'source binary version'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\windows-x86_64'
        Ensure-Directory $resourcesDir
        Copy-Item -LiteralPath $bin -Destination $dest -Force
        Copy-Item -LiteralPath $sandbox -Destination $sandboxDest -Force
        Invoke-Checked { & $dest --version } 'destination binary version'
        Print-FileStats $dest
        break
    }

    'build-win-x86_64-debug' {
        $bin = Join-Path $Repo 'codex-rs\target\release\anzoth.exe'
        $dest = 'C:\ai\anzoth-cli\releases\windows-x86_64-debug\anzoth.exe'
        Write-Host 'DEBUG-SYMBOL RELEASE BUILD'
        Refresh-Repo
        Invoke-Checked { cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth } 'cargo build'
        Invoke-Checked { & $bin --version } 'source binary version'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\windows-x86_64-debug'
        Copy-Item -LiteralPath $bin -Destination $dest -Force
        Invoke-Checked { & $dest --version } 'destination binary version'
        Print-FileStats $dest
        break
    }

    'build-linux-x86_64' {
        $RemoteHost = 'anzoth-dev'
        $dest = 'C:\ai\anzoth-cli\releases\linux-x86_64\anzoth'
        Write-Host 'PRODUCTION RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-linux-validation; echo PRODUCTION RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; export CARGO_PROFILE_RELEASE_DEBUG=0; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; mkdir -p dist/linux-x64; cp codex-rs/target/release/anzoth dist/linux-x64/anzoth; strip dist/linux-x64/anzoth; ./dist/linux-x64/anzoth --version; ls -lh dist/linux-x64/anzoth; sha256sum dist/linux-x64/anzoth; file dist/linux-x64/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\linux-x86_64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-linux-x86_64-fast' {
        $RemoteHost = 'anzoth-dev'
        $dest = 'C:\ai\anzoth-cli\releases\linux-x86_64\anzoth'
        Write-Host 'FAST DEVELOPMENT BUILD'
        $remote = 'set -e; cd ~/anzoth-linux-validation; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; mkdir -p dist/linux-x64-fast; cp codex-rs/target/fast-release/anzoth dist/linux-x64-fast/anzoth; ./dist/linux-x64-fast/anzoth --version; ls -lh dist/linux-x64-fast/anzoth; sha256sum dist/linux-x64-fast/anzoth; file dist/linux-x64-fast/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\linux-x86_64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64-fast/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-linux-x86_64-debug' {
        $RemoteHost = 'anzoth-dev'
        $dest = 'C:\ai\anzoth-cli\releases\linux-x86_64-debug\anzoth'
        Write-Host 'DEBUG-SYMBOL RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-linux-validation; echo DEBUG-SYMBOL RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; mkdir -p dist/linux-x64-debug; cp codex-rs/target/release/anzoth dist/linux-x64-debug/anzoth; ./dist/linux-x64-debug/anzoth --version; ls -lh dist/linux-x64-debug/anzoth; sha256sum dist/linux-x64-debug/anzoth; file dist/linux-x64-debug/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\linux-x86_64-debug'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64-debug/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-macos-x86_64' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-x86_64\anzoth'
        Write-Host 'PRODUCTION RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo PRODUCTION RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; export CARGO_PROFILE_RELEASE_DEBUG=0; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target x86_64-apple-darwin; strip codex-rs/target/x86_64-apple-darwin/release/anzoth; ./codex-rs/target/x86_64-apple-darwin/release/anzoth --version; ls -lh codex-rs/target/x86_64-apple-darwin/release/anzoth; shasum -a 256 codex-rs/target/x86_64-apple-darwin/release/anzoth; file codex-rs/target/x86_64-apple-darwin/release/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-x86_64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/release/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-macos-x86_64-fast' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-x86_64\anzoth'
        Write-Host 'FAST DEVELOPMENT BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target x86_64-apple-darwin; ./codex-rs/target/x86_64-apple-darwin/fast-release/anzoth --version; ls -lh codex-rs/target/x86_64-apple-darwin/fast-release/anzoth; shasum -a 256 codex-rs/target/x86_64-apple-darwin/fast-release/anzoth; file codex-rs/target/x86_64-apple-darwin/fast-release/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-x86_64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/fast-release/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-macos-x86_64-debug' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-x86_64-debug\anzoth'
        Write-Host 'DEBUG-SYMBOL RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo DEBUG-SYMBOL RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target x86_64-apple-darwin; ./codex-rs/target/x86_64-apple-darwin/release/anzoth --version; ls -lh codex-rs/target/x86_64-apple-darwin/release/anzoth; shasum -a 256 codex-rs/target/x86_64-apple-darwin/release/anzoth; file codex-rs/target/x86_64-apple-darwin/release/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-x86_64-debug'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/release/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-macos-arm64-on-intel' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-arm64\anzoth'
        Write-Host 'PRODUCTION RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo PRODUCTION RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; export CARGO_PROFILE_RELEASE_DEBUG=0; rustup target add aarch64-apple-darwin; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target aarch64-apple-darwin; strip codex-rs/target/aarch64-apple-darwin/release/anzoth; ls -lh codex-rs/target/aarch64-apple-darwin/release/anzoth; shasum -a 256 codex-rs/target/aarch64-apple-darwin/release/anzoth; file codex-rs/target/aarch64-apple-darwin/release/anzoth; lipo -info codex-rs/target/aarch64-apple-darwin/release/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-arm64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/release/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        Write-Host 'NOTE: Runtime-check this ARM64 binary on mac-m1 before release.'
        break
    }

    'build-macos-arm64-on-intel-fast' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-arm64\anzoth'
        Write-Host 'FAST DEVELOPMENT BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; rustup target add aarch64-apple-darwin; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target aarch64-apple-darwin; ./codex-rs/target/aarch64-apple-darwin/fast-release/anzoth --version; ls -lh codex-rs/target/aarch64-apple-darwin/fast-release/anzoth; shasum -a 256 codex-rs/target/aarch64-apple-darwin/fast-release/anzoth; file codex-rs/target/aarch64-apple-darwin/fast-release/anzoth; lipo -info codex-rs/target/aarch64-apple-darwin/fast-release/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-arm64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/fast-release/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-macos-arm64-on-intel-debug' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-arm64-debug\anzoth'
        Write-Host 'DEBUG-SYMBOL RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo DEBUG-SYMBOL RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; rustup target add aarch64-apple-darwin; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target aarch64-apple-darwin; ls -lh codex-rs/target/aarch64-apple-darwin/release/anzoth; shasum -a 256 codex-rs/target/aarch64-apple-darwin/release/anzoth; file codex-rs/target/aarch64-apple-darwin/release/anzoth; lipo -info codex-rs/target/aarch64-apple-darwin/release/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-arm64-debug'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/release/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-macos-arm64-on-m1' {
        & "$PSScriptRoot\build-macos-arm64.ps1" -Profile release
        break
        $RemoteHost = 'mac-m1'
        $dest = 'C:\ai\anzoth-cli\releases\macos-arm64\anzoth'
        Write-Host 'PRODUCTION RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo PRODUCTION RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; export CARGO_PROFILE_RELEASE_DEBUG=0; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; strip codex-rs/target/release/anzoth; ./codex-rs/target/release/anzoth --version; ls -lh codex-rs/target/release/anzoth; shasum -a 256 codex-rs/target/release/anzoth; file codex-rs/target/release/anzoth; lipo -info codex-rs/target/release/anzoth; if ! file codex-rs/target/release/anzoth | grep -q "arm64"; then echo "ERROR: macOS ARM64 build did not resolve to arm64." >&2; exit 1; fi; if file codex-rs/target/release/anzoth | grep -q "x86_64"; then echo "ERROR: macOS ARM64 build resolved to x86_64." >&2; exit 1; fi; if ! lipo -info codex-rs/target/release/anzoth | grep -q "arm64"; then echo "ERROR: lipo did not report arm64 for macOS ARM64 build." >&2; exit 1; fi'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-arm64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/release/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-macos-arm64-on-m1-fast' {
        & "$PSScriptRoot\build-macos-arm64.ps1" -Profile fast
        break
        $RemoteHost = 'mac-m1'
        $dest = 'C:\ai\anzoth-cli\releases\macos-arm64\anzoth'
        Write-Host 'FAST DEVELOPMENT BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; ./codex-rs/target/fast-release/anzoth --version; ls -lh codex-rs/target/fast-release/anzoth; shasum -a 256 codex-rs/target/fast-release/anzoth; file codex-rs/target/fast-release/anzoth; lipo -info codex-rs/target/fast-release/anzoth; if ! file codex-rs/target/fast-release/anzoth | grep -q "arm64"; then echo "ERROR: macOS ARM64 build did not resolve to arm64." >&2; exit 1; fi; if file codex-rs/target/fast-release/anzoth | grep -q "x86_64"; then echo "ERROR: macOS ARM64 build resolved to x86_64." >&2; exit 1; fi; if ! lipo -info codex-rs/target/fast-release/anzoth | grep -q "arm64"; then echo "ERROR: lipo did not report arm64 for macOS ARM64 build." >&2; exit 1; fi'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-arm64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/fast-release/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }

    'build-macos-arm64-on-m1-debug' {
        & "$PSScriptRoot\build-macos-arm64.ps1" -Profile debug
        break
        $RemoteHost = 'mac-m1'
        $dest = 'C:\ai\anzoth-cli\releases\macos-arm64-debug\anzoth'
        Write-Host 'DEBUG-SYMBOL RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo DEBUG-SYMBOL RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; ls -lh codex-rs/target/release/anzoth; shasum -a 256 codex-rs/target/release/anzoth; file codex-rs/target/release/anzoth; lipo -info codex-rs/target/release/anzoth; if ! file codex-rs/target/release/anzoth | grep -q "arm64"; then echo "ERROR: macOS ARM64 build did not resolve to arm64." >&2; exit 1; fi; if file codex-rs/target/release/anzoth | grep -q "x86_64"; then echo "ERROR: macOS ARM64 build resolved to x86_64." >&2; exit 1; fi; if ! lipo -info codex-rs/target/release/anzoth | grep -q "arm64"; then echo "ERROR: lipo did not report arm64 for macOS ARM64 build." >&2; exit 1; fi'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-arm64-debug'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/release/anzoth" $dest } 'scp build artifact'
        Print-FileStats $dest
        break
    }
}
