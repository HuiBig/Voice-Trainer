#!/usr/bin/env bash

voice_trainer_macos_platform() {
  case "$(uname -m)" in
    arm64) printf '%s\n' "macos-arm64" ;;
    x86_64) printf '%s\n' "macos-x64" ;;
    *) printf 'Unsupported macOS architecture: %s\n' "$(uname -m)" >&2; return 1 ;;
  esac
}
