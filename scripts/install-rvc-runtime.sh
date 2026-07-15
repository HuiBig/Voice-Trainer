#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/macos-platform.sh"
PLATFORM="$(voice_trainer_macos_platform)"
RUNTIME_ROOT="${VOICE_TRAINER_RUNTIME_ROOT:-$PROJECT_ROOT/runtime-local/$PLATFORM}"
PYTHON="${VOICE_TRAINER_PYTHON:-$RUNTIME_ROOT/python/bin/python3}"
WHEEL_DIRECTORY="${VOICE_TRAINER_WHEEL_DIR:-$RUNTIME_ROOT/wheels/rvc-v2}"
REQUIREMENT_PATH="$PROJECT_ROOT/runtime-lock/$PLATFORM-cpu/rvc-v2-requirements.txt"
REPORT_PATH="$RUNTIME_ROOT/rvc-v2-install-report.json"

[[ "$(uname -s)" == "Darwin" ]] || { echo "This script only supports macOS." >&2; exit 1; }
[[ -x "$PYTHON" ]] || { echo "Python runtime not found: $PYTHON" >&2; exit 1; }
[[ -d "$WHEEL_DIRECTORY" ]] || { echo "RVC wheel directory not found: $WHEEL_DIRECTORY" >&2; exit 1; }

"$PYTHON" -m pip install \
  --disable-pip-version-check \
  --no-index \
  --only-binary=:all: \
  --find-links "$WHEEL_DIRECTORY" \
  --report "$REPORT_PATH" \
  --requirement "$REQUIREMENT_PATH"

"$PYTHON" -c 'import json, fairseq, librosa, numba, numpy, scipy, soundfile, torch; print(json.dumps({"torch": torch.__version__, "numpy": numpy.__version__, "librosa": librosa.__version__, "numba": numba.__version__, "scipy": scipy.__version__, "soundfile": soundfile.__version__, "fairseq": fairseq.__version__}))'
