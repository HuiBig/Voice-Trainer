[CmdletBinding()]
param(
    [string]$InstallerPath,
    [string]$TorchWheelPath,
    [string]$RuntimePath,
    [string]$BuildPythonPath,
    [string]$ConstraintPath
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$runtimeRoot = Join-Path $projectRoot "runtime-local\windows-x64"

if (-not $InstallerPath) {
    $InstallerPath = Join-Path $runtimeRoot "downloads\python-3.11.9-amd64.exe"
}
if (-not $TorchWheelPath) {
    $TorchWheelPath = Join-Path $runtimeRoot "downloads\torch-2.7.1+cpu-cp311-cp311-win_amd64.whl"
}
if (-not $RuntimePath) {
    $RuntimePath = Join-Path $runtimeRoot "python"
}
if (-not $BuildPythonPath) {
    $BuildPythonPath = Join-Path $runtimeRoot "python-build"
}
if (-not $ConstraintPath) {
    $ConstraintPath = Join-Path $projectRoot "runtime-lock\windows-x64-cpu\pytorch-2.7.1-constraints.txt"
}

foreach ($requiredFile in @($InstallerPath, $TorchWheelPath, $ConstraintPath)) {
    if (-not (Test-Path -LiteralPath $requiredFile -PathType Leaf)) {
        throw "Required file not found: $requiredFile"
    }
}

$portablePython = Join-Path $RuntimePath "python.exe"
if (-not (Test-Path -LiteralPath $portablePython -PathType Leaf)) {
    throw "Portable Python is not initialized. Run scripts/setup-python-runtime.ps1 first."
}

$buildPython = Join-Path $BuildPythonPath "python.exe"
if (-not (Test-Path -LiteralPath $buildPython -PathType Leaf)) {
    New-Item -ItemType Directory -Path $BuildPythonPath -Force | Out-Null
    $installerArguments = @(
        "/quiet"
        "InstallAllUsers=0"
        "TargetDir=`"$BuildPythonPath`""
        "Include_pip=1"
        "Include_launcher=0"
        "Include_test=0"
        "Include_doc=0"
        "Include_tcltk=0"
        "Shortcuts=0"
        "AssociateFiles=0"
        "PrependPath=0"
        "AppendPath=0"
    )
    $installer = Start-Process -FilePath $InstallerPath -ArgumentList $installerArguments -Wait -PassThru
    if ($installer.ExitCode -ne 0) {
        throw "Python build environment installer failed with exit code $($installer.ExitCode)"
    }
}

if (-not (Test-Path -LiteralPath $buildPython -PathType Leaf)) {
    throw "Build Python was not created: $buildPython"
}

$sitePackages = Join-Path $RuntimePath "Lib\site-packages"
New-Item -ItemType Directory -Path $sitePackages -Force | Out-Null
$installReport = Join-Path $runtimeRoot "pytorch-install-report.json"

& $buildPython -m pip install `
    --disable-pip-version-check `
    --upgrade `
    --only-binary=:all: `
    --constraint $ConstraintPath `
    --target $sitePackages `
    --report $installReport `
    $TorchWheelPath
if ($LASTEXITCODE -ne 0) {
    throw "PyTorch dependency installation failed with exit code $LASTEXITCODE"
}

$selfCheck = & $portablePython -c "import json, torch; print(json.dumps({'torch': torch.__version__, 'cudaAvailable': torch.cuda.is_available(), 'device': 'cpu'}))"
if ($LASTEXITCODE -ne 0) {
    throw "Portable PyTorch self-check failed with exit code $LASTEXITCODE"
}

Write-Host "Portable PyTorch runtime is ready: $sitePackages"
Write-Host $selfCheck
