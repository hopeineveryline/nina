#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v nix >/dev/null 2>&1; then
  echo "nix is required for the vm smoke check" >&2
  exit 1
fi

cd "$ROOT"
exec nix build .#checks.x86_64-linux.nixos-vm-smoke
