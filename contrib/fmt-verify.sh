#!/usr/bin/env bash
#
# The `verify` crate is not part of the workspace and cannot be formatted
# with workspace-level `cargo fmt`.

set -euox pipefail

REPO_DIR=$(git rev-parse --show-toplevel)
NIGHTLY=$(cargo metadata --no-deps --manifest-path "$REPO_DIR/Cargo.toml" --format-version 1 | jq -re '.metadata.rbmt.toolchains.nightly // .workspace_metadata.rbmt.toolchains.nightly')

cargo +"$NIGHTLY" fmt \
      --manifest-path "$REPO_DIR/verify/Cargo.toml" \
      --all -- --check
