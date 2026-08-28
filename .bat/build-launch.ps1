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

function Build-PackageResources {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Target,
        [Parameter(Mandatory = $true)]
        [string]$Profile,
        [Parameter(Mandatory = $true)]
        [string]$PackageDir,
        [Parameter(Mandatory = $true)]
        [string]$EntryPointBin,
        [Parameter(Mandatory = $true)]
        [string]$CodeModeHostBin,
        [string]$BwrapBin = $null,
        [string]$CodexCommandRunnerBin = $null,
        [string]$CodexWindowsSandboxSetupBin = $null
    )

    $args = @(
        'scripts\build_codex_package.py',
        '--target', $Target,
        '--variant', 'anzoth',
        '--cargo-profile', $Profile,
        '--package-dir', $PackageDir,
        '--force',
        '--entrypoint-bin', $EntryPointBin,
        '--code-mode-host-bin', $CodeModeHostBin
    )
    if ($BwrapBin) {
        $args += @('--bwrap-bin', $BwrapBin)
    }
    if ($CodexCommandRunnerBin) {
        $args += @('--codex-command-runner-bin', $CodexCommandRunnerBin)
    }
    if ($CodexWindowsSandboxSetupBin) {
        $args += @('--codex-windows-sandbox-setup-bin', $CodexWindowsSandboxSetupBin)
    }

    Invoke-Checked { & python @args } 'package builder'
}

Set-Location -LiteralPath $Repo

switch ($ScriptName) {
    'build-win-x86_64' {
        $bin = Join-Path $Repo 'codex-rs\target\release\anzoth.exe'
        $codeModeHost = Join-Path $Repo 'codex-rs\target\release\codex-code-mode-host.exe'
        $commandRunner = Join-Path $Repo 'codex-rs\target\release\codex-command-runner.exe'
        $sandbox = Join-Path $Repo 'codex-rs\target\release\codex-windows-sandbox-setup.exe'
        $packageDir = Join-Path $Repo 'codex-rs\target\release\anzoth-package'
        $dest = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth.exe'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\windows-x86_64\codex-code-mode-host.exe'
        $resourcesDir = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth-resources'
        $pathDir = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth-path'
        $commandRunnerDest = Join-Path $resourcesDir 'codex-command-runner.exe'
        $sandboxDest = Join-Path $resourcesDir 'codex-windows-sandbox-setup.exe'
        $rgDest = Join-Path $pathDir 'rg.exe'
        Write-Host 'PRODUCTION RELEASE BUILD'
        Refresh-Repo
        Invoke-Checked { cargo build --release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth } 'cargo build'
        Invoke-Checked { cargo build --release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host } 'cargo build code mode host'
        Invoke-Checked { cargo build --release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-windows-sandbox --bin codex-command-runner } 'cargo build command runner'
        Invoke-Checked { cargo build --release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-windows-sandbox --bin codex-windows-sandbox-setup } 'cargo build sandbox helper'
        Invoke-Checked { & $bin --version } 'source binary version'
        Build-PackageResources -Target 'x86_64-pc-windows-msvc' -Profile 'release' -PackageDir $packageDir -EntryPointBin $bin -CodeModeHostBin $codeModeHost -CodexCommandRunnerBin $commandRunner -CodexWindowsSandboxSetupBin $sandbox
        Ensure-Directory 'C:\ai\anzoth-cli\releases\windows-x86_64'
        Copy-Item -LiteralPath $codeModeHost -Destination $codeModeHostDest -Force
        Ensure-Directory $resourcesDir
        Ensure-Directory $pathDir
        Copy-Item -LiteralPath $bin -Destination $dest -Force
        Copy-Item -LiteralPath $commandRunner -Destination $commandRunnerDest -Force
        Copy-Item -LiteralPath $sandbox -Destination $sandboxDest -Force
        Copy-Item -LiteralPath (Join-Path $packageDir 'anzoth-path\rg.exe') -Destination $rgDest -Force
        Invoke-Checked { & $dest --version } 'destination binary version'
        Print-FileStats $dest
        break
    }

    'build-win-x86_64-fast' {
        $bin = Join-Path $Repo 'codex-rs\target\fast-release\anzoth.exe'
        $codeModeHost = Join-Path $Repo 'codex-rs\target\fast-release\codex-code-mode-host.exe'
        $commandRunner = Join-Path $Repo 'codex-rs\target\fast-release\codex-command-runner.exe'
        $sandbox = Join-Path $Repo 'codex-rs\target\fast-release\codex-windows-sandbox-setup.exe'
        $packageDir = Join-Path $Repo 'codex-rs\target\fast-release\anzoth-package'
        $dest = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth.exe'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\windows-x86_64\codex-code-mode-host.exe'
        $resourcesDir = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth-resources'
        $pathDir = 'C:\ai\anzoth-cli\releases\windows-x86_64\anzoth-path'
        $commandRunnerDest = Join-Path $resourcesDir 'codex-command-runner.exe'
        $sandboxDest = Join-Path $resourcesDir 'codex-windows-sandbox-setup.exe'
        $rgDest = Join-Path $pathDir 'rg.exe'
        Write-Host 'FAST DEVELOPMENT BUILD'
        Refresh-Repo
        Invoke-Checked { cargo build --profile fast-release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth } 'cargo build'
        Invoke-Checked { cargo build --profile fast-release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host } 'cargo build code mode host'
        Invoke-Checked { cargo build --profile fast-release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-windows-sandbox --bin codex-command-runner } 'cargo build command runner'
        Invoke-Checked { cargo build --profile fast-release -j 20 --manifest-path codex-rs/Cargo.toml -p codex-windows-sandbox --bin codex-windows-sandbox-setup } 'cargo build sandbox helper'
        Invoke-Checked { & $bin --version } 'source binary version'
        Build-PackageResources -Target 'x86_64-pc-windows-msvc' -Profile 'fast-release' -PackageDir $packageDir -EntryPointBin $bin -CodeModeHostBin $codeModeHost -CodexCommandRunnerBin $commandRunner -CodexWindowsSandboxSetupBin $sandbox
        Ensure-Directory 'C:\ai\anzoth-cli\releases\windows-x86_64'
        Copy-Item -LiteralPath $codeModeHost -Destination $codeModeHostDest -Force
        Ensure-Directory $resourcesDir
        Ensure-Directory $pathDir
        Copy-Item -LiteralPath $bin -Destination $dest -Force
        Copy-Item -LiteralPath $commandRunner -Destination $commandRunnerDest -Force
        Copy-Item -LiteralPath $sandbox -Destination $sandboxDest -Force
        Copy-Item -LiteralPath (Join-Path $packageDir 'anzoth-path\rg.exe') -Destination $rgDest -Force
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
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\linux-x86_64\codex-code-mode-host'
        $pathDir = 'C:\ai\anzoth-cli\releases\linux-x86_64\anzoth-path'
        $resourcesDir = 'C:\ai\anzoth-cli\releases\linux-x86_64\anzoth-resources'
        $packageDir = '~/anzoth-linux-validation/dist/linux-x64/package'
        Write-Host 'PRODUCTION RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-linux-validation; echo PRODUCTION RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; export CARGO_PROFILE_RELEASE_DEBUG=0; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-bwrap --bin bwrap; mkdir -p dist/linux-x64; cp codex-rs/target/release/anzoth dist/linux-x64/anzoth; cp codex-rs/target/release/codex-code-mode-host dist/linux-x64/codex-code-mode-host; cp codex-rs/target/release/bwrap dist/linux-x64/bwrap; strip dist/linux-x64/anzoth dist/linux-x64/codex-code-mode-host dist/linux-x64/bwrap; python3 scripts/build_codex_package.py --target x86_64-unknown-linux-gnu --variant anzoth --cargo-profile release --package-dir dist/linux-x64/package --force --entrypoint-bin dist/linux-x64/anzoth --code-mode-host-bin dist/linux-x64/codex-code-mode-host --bwrap-bin dist/linux-x64/bwrap; ./dist/linux-x64/anzoth --version; ls -lh dist/linux-x64/anzoth dist/linux-x64/codex-code-mode-host dist/linux-x64/bwrap dist/linux-x64/package/anzoth-path/rg dist/linux-x64/package/anzoth-resources/bwrap; sha256sum dist/linux-x64/anzoth dist/linux-x64/codex-code-mode-host dist/linux-x64/bwrap dist/linux-x64/package/anzoth-path/rg dist/linux-x64/package/anzoth-resources/bwrap; file dist/linux-x64/anzoth dist/linux-x64/codex-code-mode-host dist/linux-x64/bwrap dist/linux-x64/package/anzoth-path/rg dist/linux-x64/package/anzoth-resources/bwrap'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\linux-x86_64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64/anzoth" $dest } 'scp build artifact'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64/codex-code-mode-host" $codeModeHostDest } 'scp code mode host'
        Ensure-Directory $pathDir
        Ensure-Directory $resourcesDir
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64/package/anzoth-path/rg" (Join-Path $pathDir 'rg') } 'scp rg'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64/package/anzoth-resources/bwrap" (Join-Path $resourcesDir 'bwrap') } 'scp bwrap'
        Print-FileStats $dest
        break
    }

    'build-linux-x86_64-fast' {
        $RemoteHost = 'anzoth-dev'
        $dest = 'C:\ai\anzoth-cli\releases\linux-x86_64\anzoth'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\linux-x86_64\codex-code-mode-host'
        $pathDir = 'C:\ai\anzoth-cli\releases\linux-x86_64\anzoth-path'
        $resourcesDir = 'C:\ai\anzoth-cli\releases\linux-x86_64\anzoth-resources'
        $packageDir = '~/anzoth-linux-validation/dist/linux-x64-fast/package'
        Write-Host 'FAST DEVELOPMENT BUILD'
        $remote = 'set -e; cd ~/anzoth-linux-validation; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-bwrap --bin bwrap; mkdir -p dist/linux-x64-fast; cp codex-rs/target/fast-release/anzoth dist/linux-x64-fast/anzoth; cp codex-rs/target/fast-release/codex-code-mode-host dist/linux-x64-fast/codex-code-mode-host; cp codex-rs/target/fast-release/bwrap dist/linux-x64-fast/bwrap; strip dist/linux-x64-fast/anzoth dist/linux-x64-fast/codex-code-mode-host dist/linux-x64-fast/bwrap; python3 scripts/build_codex_package.py --target x86_64-unknown-linux-gnu --variant anzoth --cargo-profile fast-release --package-dir dist/linux-x64-fast/package --force --entrypoint-bin dist/linux-x64-fast/anzoth --code-mode-host-bin dist/linux-x64-fast/codex-code-mode-host --bwrap-bin dist/linux-x64-fast/bwrap; ./dist/linux-x64-fast/anzoth --version; ls -lh dist/linux-x64-fast/anzoth dist/linux-x64-fast/codex-code-mode-host dist/linux-x64-fast/bwrap dist/linux-x64-fast/package/anzoth-path/rg dist/linux-x64-fast/package/anzoth-resources/bwrap; sha256sum dist/linux-x64-fast/anzoth dist/linux-x64-fast/codex-code-mode-host dist/linux-x64-fast/bwrap dist/linux-x64-fast/package/anzoth-path/rg dist/linux-x64-fast/package/anzoth-resources/bwrap; file dist/linux-x64-fast/anzoth dist/linux-x64-fast/codex-code-mode-host dist/linux-x64-fast/bwrap dist/linux-x64-fast/package/anzoth-path/rg dist/linux-x64-fast/package/anzoth-resources/bwrap'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\linux-x86_64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64-fast/anzoth" $dest } 'scp build artifact'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64-fast/codex-code-mode-host" $codeModeHostDest } 'scp code mode host'
        Ensure-Directory $pathDir
        Ensure-Directory $resourcesDir
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64-fast/package/anzoth-path/rg" (Join-Path $pathDir 'rg') } 'scp rg'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64-fast/package/anzoth-resources/bwrap" (Join-Path $resourcesDir 'bwrap') } 'scp bwrap'
        Print-FileStats $dest
        break
    }

    'build-linux-x86_64-debug' {
        $RemoteHost = 'anzoth-dev'
        $dest = 'C:\ai\anzoth-cli\releases\linux-x86_64-debug\anzoth'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\linux-x86_64-debug\codex-code-mode-host'
        Write-Host 'DEBUG-SYMBOL RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-linux-validation; echo DEBUG-SYMBOL RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host; mkdir -p dist/linux-x64-debug; cp codex-rs/target/release/anzoth dist/linux-x64-debug/anzoth; cp codex-rs/target/release/codex-code-mode-host dist/linux-x64-debug/codex-code-mode-host; ./dist/linux-x64-debug/anzoth --version; ls -lh dist/linux-x64-debug/anzoth dist/linux-x64-debug/codex-code-mode-host; sha256sum dist/linux-x64-debug/anzoth dist/linux-x64-debug/codex-code-mode-host; file dist/linux-x64-debug/anzoth dist/linux-x64-debug/codex-code-mode-host'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\linux-x86_64-debug'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64-debug/anzoth" $dest } 'scp build artifact'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-linux-validation/dist/linux-x64-debug/codex-code-mode-host" $codeModeHostDest } 'scp code mode host'
        Print-FileStats $dest
        break
    }

    'build-macos-x86_64' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-x86_64\anzoth'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\macos-x86_64\codex-code-mode-host'
        $pathDir = 'C:\ai\anzoth-cli\releases\macos-x86_64\anzoth-path'
        $packageDir = '~/anzoth-mac-validation/dist/macos-x64/package'
        Write-Host 'PRODUCTION RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo PRODUCTION RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; export CARGO_PROFILE_RELEASE_DEBUG=0; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target x86_64-apple-darwin; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host --target x86_64-apple-darwin; mkdir -p dist/macos-x64; cp codex-rs/target/x86_64-apple-darwin/release/anzoth dist/macos-x64/anzoth; cp codex-rs/target/x86_64-apple-darwin/release/codex-code-mode-host dist/macos-x64/codex-code-mode-host; strip dist/macos-x64/anzoth dist/macos-x64/codex-code-mode-host; python3 scripts/build_codex_package.py --target x86_64-apple-darwin --variant anzoth --cargo-profile release --package-dir dist/macos-x64/package --force --entrypoint-bin dist/macos-x64/anzoth --code-mode-host-bin dist/macos-x64/codex-code-mode-host; ./dist/macos-x64/anzoth --version; ls -lh dist/macos-x64/anzoth dist/macos-x64/codex-code-mode-host dist/macos-x64/package/anzoth-path/rg; shasum -a 256 dist/macos-x64/anzoth dist/macos-x64/codex-code-mode-host dist/macos-x64/package/anzoth-path/rg; file dist/macos-x64/anzoth dist/macos-x64/codex-code-mode-host dist/macos-x64/package/anzoth-path/rg'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-x86_64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/release/anzoth" $dest } 'scp build artifact'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/release/codex-code-mode-host" $codeModeHostDest } 'scp code mode host'
        Ensure-Directory $pathDir
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/dist/macos-x64/package/anzoth-path/rg" (Join-Path $pathDir 'rg') } 'scp rg'
        Print-FileStats $dest
        break
    }

    'build-macos-x86_64-fast' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-x86_64\anzoth'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\macos-x86_64\codex-code-mode-host'
        $pathDir = 'C:\ai\anzoth-cli\releases\macos-x86_64\anzoth-path'
        $packageDir = '~/anzoth-mac-validation/dist/macos-x64-fast/package'
        Write-Host 'FAST DEVELOPMENT BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target x86_64-apple-darwin; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host --target x86_64-apple-darwin; mkdir -p dist/macos-x64-fast; cp codex-rs/target/x86_64-apple-darwin/fast-release/anzoth dist/macos-x64-fast/anzoth; cp codex-rs/target/x86_64-apple-darwin/fast-release/codex-code-mode-host dist/macos-x64-fast/codex-code-mode-host; python3 scripts/build_codex_package.py --target x86_64-apple-darwin --variant anzoth --cargo-profile fast-release --package-dir dist/macos-x64-fast/package --force --entrypoint-bin dist/macos-x64-fast/anzoth --code-mode-host-bin dist/macos-x64-fast/codex-code-mode-host; ./dist/macos-x64-fast/anzoth --version; ls -lh dist/macos-x64-fast/anzoth dist/macos-x64-fast/codex-code-mode-host dist/macos-x64-fast/package/anzoth-path/rg; shasum -a 256 dist/macos-x64-fast/anzoth dist/macos-x64-fast/codex-code-mode-host dist/macos-x64-fast/package/anzoth-path/rg; file dist/macos-x64-fast/anzoth dist/macos-x64-fast/codex-code-mode-host dist/macos-x64-fast/package/anzoth-path/rg'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-x86_64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/fast-release/anzoth" $dest } 'scp build artifact'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/fast-release/codex-code-mode-host" $codeModeHostDest } 'scp code mode host'
        Ensure-Directory $pathDir
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/dist/macos-x64-fast/package/anzoth-path/rg" (Join-Path $pathDir 'rg') } 'scp rg'
        Print-FileStats $dest
        break
    }

    'build-macos-x86_64-debug' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-x86_64-debug\anzoth'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\macos-x86_64-debug\codex-code-mode-host'
        Write-Host 'DEBUG-SYMBOL RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo DEBUG-SYMBOL RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target x86_64-apple-darwin; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host --target x86_64-apple-darwin; ./codex-rs/target/x86_64-apple-darwin/release/anzoth --version; ls -lh codex-rs/target/x86_64-apple-darwin/release/anzoth codex-rs/target/x86_64-apple-darwin/release/codex-code-mode-host; shasum -a 256 codex-rs/target/x86_64-apple-darwin/release/anzoth codex-rs/target/x86_64-apple-darwin/release/codex-code-mode-host; file codex-rs/target/x86_64-apple-darwin/release/anzoth codex-rs/target/x86_64-apple-darwin/release/codex-code-mode-host'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-x86_64-debug'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/release/anzoth" $dest } 'scp build artifact'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/x86_64-apple-darwin/release/codex-code-mode-host" $codeModeHostDest } 'scp code mode host'
        Print-FileStats $dest
        break
    }

    'build-macos-arm64-on-intel' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-arm64\anzoth'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\macos-arm64\codex-code-mode-host'
        $pathDir = 'C:\ai\anzoth-cli\releases\macos-arm64\anzoth-path'
        $packageDir = '~/anzoth-mac-validation/dist/macos-arm64/package'
        Write-Host 'PRODUCTION RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo PRODUCTION RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; export CARGO_PROFILE_RELEASE_DEBUG=0; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; rustup target add aarch64-apple-darwin; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target aarch64-apple-darwin; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host --target aarch64-apple-darwin; mkdir -p dist/macos-arm64; cp codex-rs/target/aarch64-apple-darwin/release/anzoth dist/macos-arm64/anzoth; cp codex-rs/target/aarch64-apple-darwin/release/codex-code-mode-host dist/macos-arm64/codex-code-mode-host; strip dist/macos-arm64/anzoth dist/macos-arm64/codex-code-mode-host; python3 scripts/build_codex_package.py --target aarch64-apple-darwin --variant anzoth --cargo-profile release --package-dir dist/macos-arm64/package --force --entrypoint-bin dist/macos-arm64/anzoth --code-mode-host-bin dist/macos-arm64/codex-code-mode-host; ls -lh dist/macos-arm64/anzoth dist/macos-arm64/codex-code-mode-host dist/macos-arm64/package/anzoth-path/rg; shasum -a 256 dist/macos-arm64/anzoth dist/macos-arm64/codex-code-mode-host dist/macos-arm64/package/anzoth-path/rg; file dist/macos-arm64/anzoth dist/macos-arm64/codex-code-mode-host dist/macos-arm64/package/anzoth-path/rg; lipo -info dist/macos-arm64/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-arm64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/release/anzoth" $dest } 'scp build artifact'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/release/codex-code-mode-host" $codeModeHostDest } 'scp code mode host'
        Ensure-Directory $pathDir
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/dist/macos-arm64/package/anzoth-path/rg" (Join-Path $pathDir 'rg') } 'scp rg'
        Print-FileStats $dest
        Write-Host 'NOTE: Runtime-check this ARM64 binary on mac-m1 before release.'
        break
    }

    'build-macos-arm64-on-intel-fast' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-arm64\anzoth'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\macos-arm64\codex-code-mode-host'
        $pathDir = 'C:\ai\anzoth-cli\releases\macos-arm64\anzoth-path'
        $packageDir = '~/anzoth-mac-validation/dist/macos-arm64-fast/package'
        Write-Host 'FAST DEVELOPMENT BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo FAST DEVELOPMENT BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; rustup target add aarch64-apple-darwin; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target aarch64-apple-darwin; cargo build --profile fast-release --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host --target aarch64-apple-darwin; mkdir -p dist/macos-arm64-fast; cp codex-rs/target/aarch64-apple-darwin/fast-release/anzoth dist/macos-arm64-fast/anzoth; cp codex-rs/target/aarch64-apple-darwin/fast-release/codex-code-mode-host dist/macos-arm64-fast/codex-code-mode-host; python3 scripts/build_codex_package.py --target aarch64-apple-darwin --variant anzoth --cargo-profile fast-release --package-dir dist/macos-arm64-fast/package --force --entrypoint-bin dist/macos-arm64-fast/anzoth --code-mode-host-bin dist/macos-arm64-fast/codex-code-mode-host; ./dist/macos-arm64-fast/anzoth --version; ls -lh dist/macos-arm64-fast/anzoth dist/macos-arm64-fast/codex-code-mode-host dist/macos-arm64-fast/package/anzoth-path/rg; shasum -a 256 dist/macos-arm64-fast/anzoth dist/macos-arm64-fast/codex-code-mode-host dist/macos-arm64-fast/package/anzoth-path/rg; file dist/macos-arm64-fast/anzoth dist/macos-arm64-fast/codex-code-mode-host dist/macos-arm64-fast/package/anzoth-path/rg; lipo -info dist/macos-arm64-fast/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-arm64'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/fast-release/anzoth" $dest } 'scp build artifact'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/fast-release/codex-code-mode-host" $codeModeHostDest } 'scp code mode host'
        Ensure-Directory $pathDir
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/dist/macos-arm64-fast/package/anzoth-path/rg" (Join-Path $pathDir 'rg') } 'scp rg'
        Print-FileStats $dest
        break
    }

    'build-macos-arm64-on-intel-debug' {
        $RemoteHost = 'mac'
        $dest = 'C:\ai\anzoth-cli\releases\macos-arm64-debug\anzoth'
        $codeModeHostDest = 'C:\ai\anzoth-cli\releases\macos-arm64-debug\codex-code-mode-host'
        $pathDir = 'C:\ai\anzoth-cli\releases\macos-arm64-debug\anzoth-path'
        $packageDir = '~/anzoth-mac-validation/dist/macos-arm64-debug/package'
        Write-Host 'DEBUG-SYMBOL RELEASE BUILD'
        $remote = 'set -e; cd ~/anzoth-mac-validation; echo DEBUG-SYMBOL RELEASE BUILD; git fetch origin; git checkout anzoth-rebrand; git pull --ff-only origin anzoth-rebrand; git rev-parse HEAD; [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"; export PATH="$HOME/.cargo/bin:$PATH"; command -v cargo; rustup target add aarch64-apple-darwin; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target aarch64-apple-darwin; cargo build --release --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host --target aarch64-apple-darwin; mkdir -p dist/macos-arm64-debug; cp codex-rs/target/aarch64-apple-darwin/release/anzoth dist/macos-arm64-debug/anzoth; cp codex-rs/target/aarch64-apple-darwin/release/codex-code-mode-host dist/macos-arm64-debug/codex-code-mode-host; python3 scripts/build_codex_package.py --target aarch64-apple-darwin --variant anzoth --cargo-profile release --package-dir dist/macos-arm64-debug/package --force --entrypoint-bin dist/macos-arm64-debug/anzoth --code-mode-host-bin dist/macos-arm64-debug/codex-code-mode-host; ls -lh dist/macos-arm64-debug/anzoth dist/macos-arm64-debug/codex-code-mode-host dist/macos-arm64-debug/package/anzoth-path/rg; shasum -a 256 dist/macos-arm64-debug/anzoth dist/macos-arm64-debug/codex-code-mode-host dist/macos-arm64-debug/package/anzoth-path/rg; file dist/macos-arm64-debug/anzoth dist/macos-arm64-debug/codex-code-mode-host dist/macos-arm64-debug/package/anzoth-path/rg; lipo -info dist/macos-arm64-debug/anzoth'
        Invoke-Checked { & ssh $RemoteHost $remote } 'ssh build'
        Ensure-Directory 'C:\ai\anzoth-cli\releases\macos-arm64-debug'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/release/anzoth" $dest } 'scp build artifact'
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/codex-rs/target/aarch64-apple-darwin/release/codex-code-mode-host" $codeModeHostDest } 'scp code mode host'
        Ensure-Directory $pathDir
        Invoke-Checked { & scp "${RemoteHost}:~/anzoth-mac-validation/dist/macos-arm64-debug/package/anzoth-path/rg" (Join-Path $pathDir 'rg') } 'scp rg'
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
