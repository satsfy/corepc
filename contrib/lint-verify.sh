#!/usr/bin/env bash
#
# The `verify` crate is not part of the workspace because it is a dev tool.

set -euox pipefail

REPO_DIR=$(git rev-parse --show-toplevel)
NIGHTLY=$(cargo metadata --no-deps --manifest-path "$REPO_DIR/Cargo.toml" --format-version 1 | jq -re '.metadata.rbmt.toolchains.nightly // .workspace_metadata.rbmt.toolchains.nightly')

cargo +"$NIGHTLY" clippy \
      --manifest-path "$REPO_DIR/verify/Cargo.toml" \
      --config ./rustfmt.toml \
      --all-targets --all-features \
      -- --deny warnings
