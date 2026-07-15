[CmdletBinding()]
param(
    [string]$ArchivePath,
    [string]$RuntimePath
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot

if (-not $ArchivePath) {
    $ArchivePath = Join-Path $projectRoot "runtime-local\windows-x64\downloads\python-3.11.9-embed-amd64.zip"
}

if (-not $RuntimePath) {
    $RuntimePath = Join-Path $projectRoot "runtime-local\windows-x64\python"
}

if (-not (Test-Path -LiteralPath $ArchivePath -PathType Leaf)) {
    throw "Python embeddable archive not found: $ArchivePath"
}

$pythonExe = Join-Path $RuntimePath "python.exe"

if (-not (Test-Path -LiteralPath $pythonExe -PathType Leaf)) {
    if (Test-Path -LiteralPath $RuntimePath) {
        $existingItems = @(Get-ChildItem -LiteralPath $RuntimePath -Force)
        if ($existingItems.Count -gt 0) {
            throw "Runtime directory is not empty and does not contain python.exe: $RuntimePath"
        }
    }

    New-Item -ItemType Directory -Path $RuntimePath -Force | Out-Null
    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $RuntimePath
}

$pthPath = Join-Path $RuntimePath "python311._pth"
if (-not (Test-Path -LiteralPath $pthPath -PathType Leaf)) {
    throw "python311._pth was not found after extraction: $pthPath"
}

$pthLines = @(
    "python311.zip"
    "."
    "Lib"
    "Lib\site-packages"
    "import site"
)
$pthLines | Set-Content -LiteralPath $pthPath -Encoding Ascii

New-Item -ItemType Directory -Path (Join-Path $RuntimePath "Lib\site-packages") -Force | Out-Null

$version = & $pythonExe -c "import json, platform, sys; print(json.dumps({'version': platform.python_version(), 'executable': sys.executable, 'architecture': platform.architecture()[0]}))"
if ($LASTEXITCODE -ne 0) {
    throw "Portable Python self-check failed with exit code $LASTEXITCODE"
}

$archiveHash = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
$manifest = [ordered]@{
    schemaVersion = 1
    pythonVersion = "3.11.9"
    platform = "windows-x64"
    sourceArchive = Split-Path -Leaf $ArchivePath
    sourceSha256 = $archiveHash
}
$manifest | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $RuntimePath "runtime-manifest.json") -Encoding UTF8

Write-Host "Portable Python runtime is ready: $RuntimePath"
Write-Host $version
