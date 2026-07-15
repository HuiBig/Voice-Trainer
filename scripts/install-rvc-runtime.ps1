[CmdletBinding()]
param(
    [string]$RuntimePath,
    [string]$BuildPythonPath,
    [string]$WheelDirectory,
    [string]$RequirementPath
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$runtimeRoot = Join-Path $projectRoot "runtime-local\windows-x64"

if (-not $RuntimePath) {
    $RuntimePath = Join-Path $runtimeRoot "python"
}
if (-not $BuildPythonPath) {
    $BuildPythonPath = Join-Path $runtimeRoot "python-build"
}
if (-not $WheelDirectory) {
    $WheelDirectory = Join-Path $runtimeRoot "wheels\rvc-v2"
}
if (-not $RequirementPath) {
    $RequirementPath = Join-Path $projectRoot "runtime-lock\rvc-v2\requirements.txt"
}

$portablePython = Join-Path $RuntimePath "python.exe"
$buildPython = Join-Path $BuildPythonPath "python.exe"
foreach ($requiredPath in @($portablePython, $buildPython, $WheelDirectory, $RequirementPath)) {
    if (-not (Test-Path -LiteralPath $requiredPath)) {
        throw "Required runtime input not found: $requiredPath"
    }
}

$sitePackages = Join-Path $RuntimePath "Lib\site-packages"
$report = Join-Path $runtimeRoot "rvc-v2-install-report.json"
& $buildPython -m pip install `
    --disable-pip-version-check `
    --no-index `
    --only-binary=:all: `
    --find-links $WheelDirectory `
    --target $sitePackages `
    --report $report `
    --requirement $RequirementPath
if ($LASTEXITCODE -ne 0) {
    throw "RVC v2 dependency installation failed with exit code $LASTEXITCODE"
}

$selfCheck = & $portablePython -c "import json, fairseq, librosa, numba, numpy, scipy, soundfile, torch; print(json.dumps({'torch': torch.__version__, 'numpy': numpy.__version__, 'librosa': librosa.__version__, 'numba': numba.__version__, 'scipy': scipy.__version__, 'soundfile': soundfile.__version__, 'fairseq': fairseq.__version__}))"
if ($LASTEXITCODE -ne 0) {
    throw "Portable RVC v2 dependency self-check failed with exit code $LASTEXITCODE"
}

Write-Host "Portable RVC v2 dependencies are ready: $sitePackages"
Write-Host $selfCheck
