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
  echo "qemu-system-x86_64 is required for the interactive guest regression" >&2
  exit 1
fi

HARNESS_ROOT="$(mktemp -d /tmp/nina-kiln-interactive-XXXXXX)"

cleanup() {
  if command -v trash >/dev/null 2>&1; then
    trash "$HARNESS_ROOT"
  fi
}

trap cleanup EXIT

mkdir -p "$HARNESS_ROOT/nina" "$HARNESS_ROOT/src"
rsync -a --exclude target --exclude .git "$ROOT"/ "$HARNESS_ROOT/nina/"

cat >"$HARNESS_ROOT/Cargo.toml" <<'EOF'
[package]
name = "nina-kiln-interactive"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF

cat >"$HARNESS_ROOT/flake.nix" <<'EOF'
{
  description = "Focused Kiln interactive harness for Nina search and install";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            pkg-config
            openssl
            expect
          ];
        };
      });
}
EOF

cat >"$HARNESS_ROOT/run-search.sh" <<'EOF'
#!/bin/sh
set -eu

home_dir=/root/nina-home
config_dir=/root/nina-config

mkdir -p "$home_dir" "$config_dir"
cat >"$home_dir/.nina.conf" <<'CONF'
editor = "true"
generations = 5
confirm = true
color = true
teach = true
animate = false

[[machines]]
name = "local"
config = "/root/nina-config"
local = true
default = true
port = 22
CONF

cat >"$config_dir/configuration.nix" <<'CONF'
{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
  ];
}
CONF

cd /root/workspace/nina
HOME="$home_dir" nix develop .#default -c cargo run -- search ripgrep
EOF

cat >"$HARNESS_ROOT/run-install.sh" <<'EOF'
#!/bin/sh
set -eu

home_dir=/root/nina-home
config_dir=/root/nina-config

mkdir -p "$home_dir" "$config_dir"
cat >"$home_dir/.nina.conf" <<'CONF'
editor = "true"
generations = 5
confirm = true
color = true
teach = true
animate = false

[[machines]]
name = "local"
config = "/root/nina-config"
local = true
default = true
port = 22
CONF

cat >"$config_dir/configuration.nix" <<'CONF'
{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
  ];
}
CONF

cd /root/workspace/nina
HOME="$home_dir" nix develop .#default -c cargo run -- install ripgrep --no-apply
EOF

cat >"$HARNESS_ROOT/src/lib.rs" <<'EOF'
use std::process::Command;

fn run_expect(script: &str) {
    let status = Command::new("expect")
        .arg("-c")
        .arg(script)
        .status()
        .expect("failed to launch expect");
    assert!(status.success(), "expect session failed with {status}");
}

#[cfg(test)]
mod tests {
    use super::run_expect;

    #[test]
    fn search_flow_uses_the_real_tui_and_launches_install() {
        run_expect(
            r#"
set timeout 240
spawn sh /root/workspace/run-search.sh
after 25000
send "i"
after 3000
send "y\r"
after 3000
send "n\r"
expect eof
"#,
        )
    }

    #[test]
    fn install_flow_accepts_the_default_pick_and_writes_config() {
        run_expect(
            r#"
set timeout 240
spawn sh /root/workspace/run-install.sh
after 25000
send "\r"
after 3000
send "y\r"
expect eof
"#,
        )
    }
}
EOF

echo "==> launching kiln interactive regression"
cmd=(
  cargo run
  --manifest-path "$KILN_ROOT/Cargo.toml"
  -p kiln-cli
  --
  vm run
  "$HARNESS_ROOT"
)

if [[ "$KEEP_ON_FAIL" == "1" ]]; then
  cmd+=(--keep-on-fail)
fi

exec "${cmd[@]}"
