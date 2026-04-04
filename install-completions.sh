#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

mkdir -p "$HOME/.local/share/bash-completion/completions"
mkdir -p "$HOME/.local/share/zsh/site-functions"
mkdir -p "$HOME/.local/share/fish/vendor_completions.d"

cp "$ROOT/completions/nina.bash" "$HOME/.local/share/bash-completion/completions/nina"
cp "$ROOT/completions/_nina" "$HOME/.local/share/zsh/site-functions/_nina"
cp "$ROOT/completions/nina.fish" "$HOME/.local/share/fish/vendor_completions.d/nina.fish"

printf 'installed nina completions into ~/.local/share for bash, zsh, and fish\n'
