#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/macos-platform.sh"
PLATFORM="$(voice_trainer_macos_platform)"
SOURCE_PATH="${VOICE_TRAINER_RVC_SOURCE:-$PROJECT_ROOT/runtime-local/$PLATFORM/rvc}"
LOCK_PATH="$PROJECT_ROOT/runtime-lock/rvc-v2/source.lock.json"
PYTHON="${VOICE_TRAINER_PYTHON:-python3}"

[[ -d "$SOURCE_PATH/.git" ]] || { echo "RVC source is not a Git checkout: $SOURCE_PATH" >&2; exit 1; }
expected_commit="$("$PYTHON" -c 'import json,sys; print(json.load(open(sys.argv[1], encoding="utf-8"))["commit"])' "$LOCK_PATH")"
actual_commit="$(git -C "$SOURCE_PATH" rev-parse HEAD)"
[[ "$actual_commit" == "$expected_commit" ]] || { echo "RVC source mismatch: expected $expected_commit, got $actual_commit" >&2; exit 1; }
echo "Verified RVC v2 source at commit $actual_commit"
