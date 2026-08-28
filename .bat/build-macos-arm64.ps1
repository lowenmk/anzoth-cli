param(
    [ValidateSet('release', 'fast', 'debug')]
    [string]$Profile = 'release'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$releaseRoot = if ($Profile -eq 'debug') {
    'C:\ai\anzoth-cli\releases\macos-arm64-debug'
} else {
    'C:\ai\anzoth-cli\releases\macos-arm64'
}
$pathRoot = Join-Path $releaseRoot 'anzoth-path'
$remoteHost = 'mac-m1'

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [scriptblock]$Script,
        [Parameter(Mandatory = $true)]
        [string]$Label
    )

    & $Script
    if ($LASTEXITCODE -ne 0) { throw "$Label failed with exit code $LASTEXITCODE" }
}

function Get-Sha256Hex([string]$Path) {
    $line = (& certutil.exe -hashfile $Path SHA256 2>$null | Select-Object -Skip 1 | Select-Object -First 1).Trim()
    if ($line -notmatch '^[0-9A-Fa-f]{64}$') { throw "ERROR: unable to compute SHA-256 for $Path" }
    $line.ToLowerInvariant()
}

function Get-ProfileSpec([string]$Profile) {
    switch ($Profile) {
        'release' { @{ CargoArgs = '--release'; TargetSubdir = 'release'; PublishLabel = 'PRODUCTION RELEASE BUILD'; Strip = $true } }
        'fast' { @{ CargoArgs = '--profile fast-release'; TargetSubdir = 'fast-release'; PublishLabel = 'FAST DEVELOPMENT BUILD'; Strip = $true } }
        'debug' { @{ CargoArgs = '--release'; TargetSubdir = 'release'; PublishLabel = 'DEBUG-SYMBOL RELEASE BUILD'; Strip = $false } }
    }
}

function Quote-Remote([string]$Value) {
    "'" + ($Value -replace "'", "'\''") + "'"
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) { throw 'ERROR: git is required.' }
if (-not (Get-Command ssh.exe -ErrorAction SilentlyContinue)) { throw 'ERROR: ssh.exe is required.' }
if (-not (Get-Command scp.exe -ErrorAction SilentlyContinue)) { throw 'ERROR: scp.exe is required.' }

$profileSpec = Get-ProfileSpec -Profile $Profile
$originUrl = (git -C $repoRoot remote get-url origin).Trim()
$sourceBranch = (git -C $repoRoot branch --show-current).Trim()
$sourceCommit = (git -C $repoRoot rev-parse HEAD).Trim()
if ([string]::IsNullOrWhiteSpace($originUrl)) { throw 'ERROR: unable to resolve origin URL.' }
if ([string]::IsNullOrWhiteSpace($sourceBranch)) { throw 'ERROR: unable to resolve current branch.' }
if ([string]::IsNullOrWhiteSpace($sourceCommit)) { throw 'ERROR: unable to resolve current HEAD.' }

$remoteArch = (& ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost 'uname -m').Trim()
if ($remoteArch -ne 'arm64') { throw "ERROR: mac-m1 reported '$remoteArch'; expected arm64." }

$remoteHome = (& ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost 'echo "$HOME"').Trim()
if ([string]::IsNullOrWhiteSpace($remoteHome)) { throw 'ERROR: unable to resolve remote home directory.' }

$remoteRepo = "$remoteHome/anzoth-cli"
$target = 'aarch64-apple-darwin'
$cargoArgs = $profileSpec.CargoArgs
$targetSubdir = $profileSpec.TargetSubdir
$publishLabel = $profileSpec.PublishLabel
$stripBinary = [bool]$profileSpec.Strip
$cargoProfile = switch ($Profile) {
    'fast' { 'fast-release' }
    default { 'release' }
}
$artifactName = 'anzoth'

Write-Host "Remote host: $remoteHost"
Write-Host "Remote repo: $remoteRepo"
Write-Host "Source branch: $sourceBranch"
Write-Host "Source HEAD: $sourceCommit"
Write-Host "Profile: $Profile"
Write-Host "Target: $target"

$repoUrlQ = Quote-Remote $originUrl
$remoteRepoQ = Quote-Remote $remoteRepo
$branchQ = Quote-Remote $sourceBranch

$remoteBootstrap = @"
set -e
if [ ! -d $remoteRepoQ/.git ]; then
  git clone --origin origin $repoUrlQ $remoteRepoQ
fi
cd $remoteRepoQ
git_status=`$(git status --short)
if [ -n "`$git_status" ]; then
  echo 'ERROR: mac-m1 checkout has local modifications:' >&2
  printf '%s\n' "`$git_status" >&2
  exit 1
fi
git fetch origin
git checkout $branchQ
git pull --ff-only origin $branchQ
git branch --show-current
git rev-parse HEAD
"@

Invoke-Checked { & ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost $remoteBootstrap } 'prepare persistent repo'

$remoteBuild = @"
set -e
cd $remoteRepoQ
echo $publishLabel
rustup target add aarch64-apple-darwin
cargo build $cargoArgs --manifest-path codex-rs/Cargo.toml -p codex-cli --bin anzoth --target aarch64-apple-darwin
cargo build $cargoArgs --manifest-path codex-rs/Cargo.toml -p codex-code-mode-host --bin codex-code-mode-host --target aarch64-apple-darwin
if [ '$stripBinary' = 'True' ]; then
  strip codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth
  strip codex-rs/target/aarch64-apple-darwin/$targetSubdir/codex-code-mode-host
fi
python3 scripts/build_codex_package.py --target aarch64-apple-darwin --variant anzoth --cargo-profile $cargoProfile --package-dir codex-rs/target/aarch64-apple-darwin/$targetSubdir/package --force --entrypoint-bin codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth --code-mode-host-bin codex-rs/target/aarch64-apple-darwin/$targetSubdir/codex-code-mode-host
./codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth --version
ls -lh codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth
ls -lh codex-rs/target/aarch64-apple-darwin/$targetSubdir/codex-code-mode-host
ls -lh codex-rs/target/aarch64-apple-darwin/$targetSubdir/package/anzoth-path/rg
shasum -a 256 codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth
shasum -a 256 codex-rs/target/aarch64-apple-darwin/$targetSubdir/codex-code-mode-host
shasum -a 256 codex-rs/target/aarch64-apple-darwin/$targetSubdir/package/anzoth-path/rg
file codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth
file codex-rs/target/aarch64-apple-darwin/$targetSubdir/codex-code-mode-host
file codex-rs/target/aarch64-apple-darwin/$targetSubdir/package/anzoth-path/rg
lipo -info codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth
if ! file codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth | grep -q 'arm64'; then
  echo 'ERROR: macOS ARM64 build did not resolve to arm64.' >&2
  exit 1
fi
if file codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth | grep -q 'x86_64'; then
  echo 'ERROR: macOS ARM64 build resolved to x86_64.' >&2
  exit 1
fi
if ! lipo -info codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth | grep -q 'arm64'; then
  echo 'ERROR: lipo did not report arm64 for macOS ARM64 build.' >&2
  exit 1
fi
"@

Invoke-Checked { & ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost $remoteBuild } 'remote ARM64 build'

$remoteArtifact = "$remoteRepo/codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth"
$remoteHostArtifact = "$remoteRepo/codex-rs/target/aarch64-apple-darwin/$targetSubdir/codex-code-mode-host"
$remoteRgArtifact = "$remoteRepo/codex-rs/target/aarch64-apple-darwin/$targetSubdir/package/anzoth-path/rg"
$version = (& ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost "cd $remoteRepoQ && './codex-rs/target/aarch64-apple-darwin/$targetSubdir/anzoth' --version").Trim()
if ($version -notmatch '^Anzoth CLI (\d+\.\d+\.\d+(?:\.\d+)?)$') { throw "ERROR: invalid version banner: $version" }
$ver = $Matches[1]

New-Item -ItemType Directory -Force -Path $releaseRoot | Out-Null
New-Item -ItemType Directory -Force -Path $pathRoot | Out-Null
$artifactName = "anzoth"
$tempArtifact = Join-Path $releaseRoot "$artifactName.tmp"
$finalArtifact = Join-Path $releaseRoot $artifactName
$tempHostArtifact = Join-Path $releaseRoot "codex-code-mode-host.tmp"
$finalHostArtifact = Join-Path $releaseRoot "codex-code-mode-host"
$tempRgArtifact = Join-Path $pathRoot "rg.tmp"
$finalRgArtifact = Join-Path $pathRoot "rg"
$manifestPath = Join-Path $releaseRoot 'release-manifest-macos-arm64.json'

Invoke-Checked { & scp.exe -o BatchMode=yes -o ConnectTimeout=15 "${remoteHost}:$remoteArtifact" $tempArtifact } 'scp build artifact'
Invoke-Checked { & scp.exe -o BatchMode=yes -o ConnectTimeout=15 "${remoteHost}:$remoteHostArtifact" $tempHostArtifact } 'scp code mode host'
Invoke-Checked { & scp.exe -o BatchMode=yes -o ConnectTimeout=15 "${remoteHost}:$remoteRgArtifact" $tempRgArtifact } 'scp rg'

$sha = Get-Sha256Hex -Path $tempArtifact
$hostSha = Get-Sha256Hex -Path $tempHostArtifact
$rgSha = Get-Sha256Hex -Path $tempRgArtifact
Set-Content -LiteralPath "$tempArtifact.sha256" -Value "$sha *$artifactName" -Encoding ASCII
Set-Content -LiteralPath "$tempHostArtifact.sha256" -Value "$hostSha *codex-code-mode-host" -Encoding ASCII
Set-Content -LiteralPath "$tempRgArtifact.sha256" -Value "$rgSha *rg" -Encoding ASCII

$remoteSha = (& ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost "shasum -a 256 $([string](Quote-Remote $remoteArtifact)) | cut -d ' ' -f 1").Trim().ToLowerInvariant()
if ($remoteSha -ne $sha) { throw "ERROR: remote SHA mismatch. Remote=$remoteSha Windows=$sha" }
$remoteHostSha = (& ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost "shasum -a 256 $([string](Quote-Remote $remoteHostArtifact)) | cut -d ' ' -f 1").Trim().ToLowerInvariant()
if ($remoteHostSha -ne $hostSha) { throw "ERROR: remote code-mode host SHA mismatch. Remote=$remoteHostSha Windows=$hostSha" }
$remoteRgSha = (& ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost "shasum -a 256 $([string](Quote-Remote $remoteRgArtifact)) | cut -d ' ' -f 1").Trim().ToLowerInvariant()
if ($remoteRgSha -ne $rgSha) { throw "ERROR: remote rg SHA mismatch. Remote=$remoteRgSha Windows=$rgSha" }

Set-Content -LiteralPath $manifestPath -Encoding UTF8 -Value (@{
    schemaVersion = 1
    platform = 'macOS'
    architecture = 'arm64'
    version = $ver
    filename = $artifactName
    target = $target
    profile = $Profile
    remoteHost = $remoteHost
    sourceBranch = $sourceBranch
    sourceCommit = $sourceCommit
    sha256 = $sha
    sourceFile = $remoteArtifact
    hostFilename = 'codex-code-mode-host'
    hostSha256 = $hostSha
    hostSourceFile = $remoteHostArtifact
    pathFilename = 'rg'
    pathSha256 = $rgSha
    pathSourceFile = $remoteRgArtifact
    releasedAt = (Get-Date).ToString('o')
} | ConvertTo-Json -Depth 6)

Move-Item -LiteralPath $tempArtifact -Destination $finalArtifact -Force
Move-Item -LiteralPath "$tempArtifact.sha256" -Destination "$finalArtifact.sha256" -Force
Move-Item -LiteralPath $tempHostArtifact -Destination $finalHostArtifact -Force
Move-Item -LiteralPath "$tempHostArtifact.sha256" -Destination "$finalHostArtifact.sha256" -Force
Move-Item -LiteralPath $tempRgArtifact -Destination $finalRgArtifact -Force
Move-Item -LiteralPath "$tempRgArtifact.sha256" -Destination "$finalRgArtifact.sha256" -Force

if ((Get-Sha256Hex -Path $finalArtifact) -ne $sha) { throw 'ERROR: local SHA mismatch after publish.' }
if ((Get-Sha256Hex -Path $finalHostArtifact) -ne $hostSha) { throw 'ERROR: local code-mode host SHA mismatch after publish.' }
if ((Get-Sha256Hex -Path $finalRgArtifact) -ne $rgSha) { throw 'ERROR: local rg SHA mismatch after publish.' }

Write-Host "Remote artifact: $remoteArtifact"
Write-Host "Remote code-mode host: $remoteHostArtifact"
Write-Host "Remote rg: $remoteRgArtifact"
Write-Host "Version: $version"
Write-Host "file: $(( & ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost "file $([string](Quote-Remote $remoteArtifact))").Trim())"
Write-Host "file (host): $(( & ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost "file $([string](Quote-Remote $remoteHostArtifact))").Trim())"
Write-Host "file (rg): $(( & ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost "file $([string](Quote-Remote $remoteRgArtifact))").Trim())"
Write-Host "lipo: $(( & ssh.exe -o BatchMode=yes -o ConnectTimeout=15 $remoteHost "lipo -info $([string](Quote-Remote $remoteArtifact))").Trim())"
Write-Host "SHA256: $sha"
Write-Host "Host SHA256: $hostSha"
Write-Host "Rg SHA256: $rgSha"
Write-Host "Published: $finalArtifact"
Write-Host "Published host: $finalHostArtifact"
Write-Host "Published rg: $finalRgArtifact"
