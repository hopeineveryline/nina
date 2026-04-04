#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KILN_ROOT="${KILN_ROOT:-$ROOT/../kiln}"

if [[ ! -d "$KILN_ROOT" ]]; then
  echo "kiln was not found at $KILN_ROOT" >&2
  echo "set KILN_ROOT or place kiln next to nina as ../kiln" >&2
  exit 1
fi

cd "$ROOT"

if command -v nix >/dev/null 2>&1; then
  exec nix develop .#kiln -c cargo run --manifest-path "$KILN_ROOT/Cargo.toml" -p kiln-cli -- fire .
fi

exec cargo run --manifest-path "$KILN_ROOT/Cargo.toml" -p kiln-cli -- fire .
