#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/macos-platform.sh"
PLATFORM="$(voice_trainer_macos_platform)"
RUNTIME_ROOT="${VOICE_TRAINER_RUNTIME_ROOT:-$PROJECT_ROOT/runtime-local/$PLATFORM}"
PYTHON="${VOICE_TRAINER_PYTHON:-$RUNTIME_ROOT/python/bin/python3}"
WHEEL_DIRECTORY="${VOICE_TRAINER_WHEEL_DIR:-$RUNTIME_ROOT/wheels/pytorch-2.7.1}"
CONSTRAINT_PATH="$PROJECT_ROOT/runtime-lock/$PLATFORM-cpu/pytorch-2.7.1-constraints.txt"

[[ "$(uname -s)" == "Darwin" ]] || { echo "This script only supports macOS." >&2; exit 1; }
[[ -x "$PYTHON" ]] || { echo "Python runtime not found: $PYTHON" >&2; exit 1; }
[[ -d "$WHEEL_DIRECTORY" ]] || { echo "PyTorch wheel directory not found: $WHEEL_DIRECTORY" >&2; exit 1; }

"$PYTHON" -m pip install \
  --disable-pip-version-check \
  --no-index \
  --only-binary=:all: \
  --find-links "$WHEEL_DIRECTORY" \
  --constraint "$CONSTRAINT_PATH" \
  torch==2.7.1 torchvision==0.22.1 torchaudio==2.7.1

"$PYTHON" -c 'import json, torch; print(json.dumps({"torch": torch.__version__, "mpsBuilt": torch.backends.mps.is_built(), "mpsAvailable": torch.backends.mps.is_available()}))'
