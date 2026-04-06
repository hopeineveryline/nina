#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="${RELEASE_REPO:-hopeineveryline/nina}"
TAG="${1:-}"

usage() {
  cat <<'EOF'
usage: scripts/release.sh <tag>

Creates or updates a GitHub release for a Nina tag with warm release notes.
EOF
}

if [[ -z "$TAG" || "$TAG" == "-h" || "$TAG" == "--help" ]]; then
  usage
  if [[ "$TAG" == "-h" || "$TAG" == "--help" ]]; then
    exit 0
  fi
  exit 1
fi

if [[ "$TAG" != v* ]]; then
  TAG="v$TAG"
fi

if ! git -C "$ROOT" rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  echo "tag $TAG does not exist locally" >&2
  exit 1
fi

current_version="$(sed -n 's/^version = "\([0-9.]*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)"
current_tag="v$current_version"
title="nina $TAG"
tmp_notes="$(mktemp -t nina-release-notes)"

cleanup() {
  if command -v trash >/dev/null 2>&1; then
    trash "$tmp_notes"
  fi
}

trap cleanup EXIT

generate_notes() {
  case "$TAG" in
    v0.4.6)
      cat <<'EOF'
This release is a softer, steadier Nina.

The big pieces here are the calmer kaomoji output, the tighter hero box, and the Kiln regression that keeps search and install honest inside a real NixOS guest. Package lookup also got less fragile, so the CLI should feel a lot less fussy now.

It is a fix-heavy release, but it is the kind of fix-heavy that makes Nina feel more like herself.
EOF
      ;;
    v0.4.7)
      cat <<'EOF'
This release keeps Nina source-only and trims away the macOS release path.

The release story is simpler now: checked-in source, local verification, and Kiln guest checks doing the heavy lifting. Less packaging noise, less drift, more focus on the NixOS toolchain Nina is built for.
EOF
      ;;
    *)
      cat <<EOF
Nina is keeping things calm in $TAG.

This release is here to make the tool feel steadier, warmer, and easier to trust. The details live in the code and the tests, but the overall shape is simple: keep the workflow gentle and the terminal experience natural.
EOF
      ;;
  esac
}

generate_notes >"$tmp_notes"

if gh release view "$TAG" --repo "$REPO" >/dev/null 2>&1; then
  gh release edit "$TAG" --repo "$REPO" --title "$title" --notes-file "$tmp_notes"
else
  create_args=(gh release create "$TAG" --repo "$REPO" --title "$title" --notes-file "$tmp_notes" --verify-tag)
  if [[ "$TAG" != "$current_tag" ]]; then
    create_args+=(--latest=false)
  fi
  "${create_args[@]}"
fi
