#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/macos-platform.sh"
PLATFORM="$(voice_trainer_macos_platform)"
case "$PLATFORM" in
  macos-arm64) EXPECTED_PYTHON_ARCH="arm64" ;;
  macos-x64) EXPECTED_PYTHON_ARCH="x86_64" ;;
esac
RUNTIME_PATH="${1:-$PROJECT_ROOT/runtime-local/$PLATFORM/python}"
BOOTSTRAP_PYTHON="${VOICE_TRAINER_BOOTSTRAP_PYTHON:-python3.11}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script only supports macOS." >&2
  exit 1
fi
if ! command -v "$BOOTSTRAP_PYTHON" >/dev/null 2>&1; then
  echo "Python 3.11 was not found. Install it first or set VOICE_TRAINER_BOOTSTRAP_PYTHON." >&2
  exit 1
fi

"$BOOTSTRAP_PYTHON" -c "import sys; assert sys.version_info[:2] == (3, 11), sys.version"
"$BOOTSTRAP_PYTHON" -c 'import platform,sys; actual=platform.machine(); expected=sys.argv[1]; assert actual == expected, f"Python architecture {actual} does not match {expected}"' "$EXPECTED_PYTHON_ARCH"
if [[ ! -x "$RUNTIME_PATH/bin/python3" ]]; then
  mkdir -p "$(dirname "$RUNTIME_PATH")"
  "$BOOTSTRAP_PYTHON" -m venv "$RUNTIME_PATH"
fi

"$RUNTIME_PATH/bin/python3" -c 'import json, pathlib, platform, sys; pathlib.Path(sys.prefix, "runtime-manifest.json").write_text(json.dumps({"schemaVersion": 1, "pythonVersion": platform.python_version(), "platform": sys.argv[1], "architecture": platform.machine(), "kind": "venv"}, indent=2) + "\n", encoding="utf-8")' "$PLATFORM"
"$RUNTIME_PATH/bin/python3" -c 'import json, platform, sys; print(json.dumps({"version": platform.python_version(), "executable": sys.executable, "architecture": platform.machine()}))'

echo "Voice Trainer Python runtime is ready: $RUNTIME_PATH"
