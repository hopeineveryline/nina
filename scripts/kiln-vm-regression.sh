#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KILN_ROOT="${KILN_ROOT:-$ROOT/../kiln}"
KEEP_ON_FAIL="${KILN_KEEP_ON_FAIL:-0}"

if [[ ! -d "$KILN_ROOT" ]]; then
  echo "kiln was not found at $KILN_ROOT" >&2
  echo "set KILN_ROOT or place kiln next to nina as ../kiln" >&2
  exit 1
fi

if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
  echo "qemu-system-x86_64 is required for the guest regression run" >&2
  exit 1
fi

cd "$ROOT"

echo "==> inventorying nina tests"
test_list="$(cargo test -- --list)"
test_count="$(printf '%s\n' "$test_list" | awk '/tests, 0 benchmarks/ { print $1 }' | tail -n1)"

if [[ -z "$test_count" ]]; then
  echo "unable to determine nina test count" >&2
  exit 1
fi

echo "nina test count: $test_count"
if (( test_count < 500 )); then
  echo "expected at least 500 tests before guest regression" >&2
  exit 1
fi

echo "==> running local verification"
cargo check
cargo build
cargo test

echo "==> launching kiln guest regression"
cmd=(
  cargo run
  --manifest-path "$KILN_ROOT/Cargo.toml"
  -p kiln-cli
  --
  vm run
  "$ROOT"
)

if [[ "$KEEP_ON_FAIL" == "1" ]]; then
  cmd+=(--keep-on-fail)
fi

exec "${cmd[@]}"
