[CmdletBinding()]
param(
    [string]$RuntimeRoot,
    [string]$LockPath
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $RuntimeRoot) {
    $RuntimeRoot = Join-Path $projectRoot "runtime-local\windows-x64"
}
if (-not $LockPath) {
    $LockPath = Join-Path $projectRoot "runtime-lock\rvc-v2\model-artifacts.lock.json"
}

$lock = Get-Content -LiteralPath $LockPath -Raw | ConvertFrom-Json
foreach ($artifact in $lock.artifacts) {
    $path = Join-Path $RuntimeRoot $artifact.destination
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Missing RVC asset '$($artifact.id)': $path"
    }
    $file = Get-Item -LiteralPath $path
    if ($file.Length -ne $artifact.size) {
        throw "Size mismatch for RVC asset '$($artifact.id)': expected $($artifact.size), got $($file.Length)"
    }
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -ne $artifact.sha256) {
        throw "SHA256 mismatch for RVC asset '$($artifact.id)'"
    }
    Write-Host "Verified $($artifact.id): $path"
}
