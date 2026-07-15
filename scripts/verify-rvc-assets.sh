#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/macos-platform.sh"
PLATFORM="$(voice_trainer_macos_platform)"
RUNTIME_ROOT="${VOICE_TRAINER_RUNTIME_ROOT:-$PROJECT_ROOT/runtime-local/$PLATFORM}"
PYTHON="${VOICE_TRAINER_PYTHON:-$RUNTIME_ROOT/python/bin/python3}"
LOCK_PATH="$PROJECT_ROOT/runtime-lock/rvc-v2/model-artifacts.lock.json"

[[ -x "$PYTHON" ]] || { echo "Python runtime not found: $PYTHON" >&2; exit 1; }
while IFS=$'\t' read -r artifact_id destination expected_size expected_hash; do
  path="$RUNTIME_ROOT/$destination"
  [[ -f "$path" ]] || { echo "Missing RVC asset '$artifact_id': $path" >&2; exit 1; }
  actual_size="$(stat -f '%z' "$path")"
  [[ "$actual_size" == "$expected_size" ]] || { echo "Size mismatch for '$artifact_id'" >&2; exit 1; }
  actual_hash="$(shasum -a 256 "$path" | awk '{print $1}')"
  [[ "$actual_hash" == "$expected_hash" ]] || { echo "SHA256 mismatch for '$artifact_id'" >&2; exit 1; }
  echo "Verified $artifact_id: $path"
done < <("$PYTHON" -c 'import json,sys; data=json.load(open(sys.argv[1], encoding="utf-8")); [print(a["id"], a["destination"], a["size"], a["sha256"], sep="\t") for a in data["artifacts"]]' "$LOCK_PATH")
