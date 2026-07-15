[CmdletBinding()]
param(
    [string]$SourcePath,
    [string]$LockPath
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $SourcePath) {
    $SourcePath = Join-Path $projectRoot "runtime-local\windows-x64\rvc"
}
if (-not $LockPath) {
    $LockPath = Join-Path $projectRoot "runtime-lock\rvc-v2\source.lock.json"
}
if (-not (Test-Path -LiteralPath (Join-Path $SourcePath ".git"))) {
    throw "RVC source is not a Git checkout: $SourcePath"
}

$lock = Get-Content -LiteralPath $LockPath -Raw | ConvertFrom-Json
$actualCommit = (& git -C $SourcePath rev-parse HEAD).Trim().ToLowerInvariant()
if ($LASTEXITCODE -ne 0) {
    throw "Unable to inspect RVC source revision: $SourcePath"
}
if ($actualCommit -ne $lock.commit) {
    throw "RVC source revision mismatch: expected $($lock.commit), got $actualCommit"
}

Write-Host "Verified RVC $($lock.modelVersion) source at commit $actualCommit"
