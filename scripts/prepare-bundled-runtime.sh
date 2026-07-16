#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

case "$(uname -s):$(uname -m)" in
  Darwin:arm64) PLATFORM="macos-arm64" ;;
  Darwin:x86_64) PLATFORM="macos-x64" ;;
  *)
    echo "Bundled runtime staging is not configured for $(uname -s) $(uname -m)." >&2
    exit 1
    ;;
esac

RUNTIME_ROOT="$PROJECT_ROOT/runtime-local/$PLATFORM"
STAGING_ROOT="$PROJECT_ROOT/src-tauri/runtime-bundle/runtime"
PYTHON="$RUNTIME_ROOT/python/bin/python3"

require_file() {
  [[ -f "$1" ]] || { echo "Missing bundled runtime file: $1" >&2; exit 1; }
}

require_executable() {
  [[ -x "$1" ]] || { echo "Missing bundled executable: $1" >&2; exit 1; }
}

require_executable "$PYTHON"
require_executable "$RUNTIME_ROOT/ffmpeg/bin/ffmpeg"
require_executable "$RUNTIME_ROOT/ffmpeg/bin/ffprobe"
require_file "$RUNTIME_ROOT/rvc/rmvpe.pt"
require_file "$RUNTIME_ROOT/rvc/assets/hubert/hubert_base.pt"
require_file "$RUNTIME_ROOT/rvc/assets/pretrained_v2/f0G40k.pth"
require_file "$RUNTIME_ROOT/rvc/assets/pretrained_v2/f0D40k.pth"

if [[ -L "$PYTHON" ]]; then
  PYTHON_TARGET="$(cd "$(dirname "$PYTHON")" && pwd -P)/$(readlink "$PYTHON")"
  case "$PYTHON_TARGET" in
    "$RUNTIME_ROOT/python/"*) ;;
    *)
      echo "Bundled Python symlink points outside the standalone runtime: $PYTHON -> $PYTHON_TARGET" >&2
      exit 1
      ;;
  esac
fi
if [[ -f "$RUNTIME_ROOT/python/pyvenv.cfg" ]]; then
  echo "A Python venv is not relocatable enough for the installer. Stage a standalone Python 3.11 distribution instead." >&2
  exit 1
fi

"$PYTHON" -c 'import sys; assert sys.version_info[:2] == (3, 11), sys.version'
"$PYTHON" -c 'import torch; assert torch.__version__.split("+")[0] == "2.7.1", torch.__version__'
"$PYTHON" -c 'import fairseq, librosa, numba, numpy, scipy, soundfile, torch'
bash "$SCRIPT_DIR/verify-rvc-assets.sh"
bash "$SCRIPT_DIR/verify-rvc-source.sh"

mkdir -p "$STAGING_ROOT"
rsync -a --delete \
  --exclude '.git/' \
  --exclude '__pycache__/' \
  --exclude '*.pyc' \
  --exclude 'wheels/' \
  --exclude 'fairseq-source/' \
  "$RUNTIME_ROOT/" "$STAGING_ROOT/"

echo "Bundled $PLATFORM runtime staged at $STAGING_ROOT"
