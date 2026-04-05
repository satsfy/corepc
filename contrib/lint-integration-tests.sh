#!/usr/bin/env bash
#
# The `integration_test` crate is not part of the workspace and cannot be built
# with `--all-features`.

set -euox pipefail

REPO_DIR=$(git rev-parse --show-toplevel)
NIGHTLY=$(cargo metadata --no-deps --manifest-path "$REPO_DIR/Cargo.toml" --format-version 1 | jq -re '.metadata.rbmt.toolchains.nightly // .workspace_metadata.rbmt.toolchains.nightly')

# Run clippy for each feature starting with an integer i.e., all
# the 'public' features.
function main() {
    pushd "$REPO_DIR/integration_test" > /dev/null

    # Extract features that start with digits from the manifest.
    features=$(grep -E '^[0-9]' Cargo.toml | grep '=' | cut -d' ' -f1)

    for feature in $features; do
        cargo +"$NIGHTLY" clippy --all-targets --features="$feature" -- -D warnings
    done

    popd
}

main "$@"
exit 0
