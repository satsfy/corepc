#!/usr/bin/env bash
#
# Update the minimal/recent lock file
# Targets with conflicting features where only specific features are used.
# Cannot be replaced by `cargo rbmt lock` because of the SPECIFIC_FEATURES_CRATES handling below.

set -euo pipefail

REPO_DIR="$(git rev-parse --show-toplevel)"

# Targets where `--all-features` is used.
ALL_FEATURE_CRATES=(bitreq client fuzz jsonrpc types verify)

SPECIFIC_FEATURES_CRATES=(integration_test bitcoind)
SPECIFIC_FEATURES=(latest)

# electrsd has mutually-exclusive version features (same shape as bitcoind) and
# cannot be checked with --all-features. Pick the highest electrs version plus
# a bitcoind version so the manifest resolves end-to-end.
ELECTRSD_FEATURES="electrs_0_10_6,bitcoind_30_2,bitcoind_download"

update_lock_files() {
    for crate in "${ALL_FEATURE_CRATES[@]}"; do
        cargo check --manifest-path "$REPO_DIR/$crate/Cargo.toml" --all-features
    done

    for crate in "${SPECIFIC_FEATURES_CRATES[@]}"; do
        cargo check --manifest-path "$REPO_DIR/$crate/Cargo.toml" --no-default-features --features="${SPECIFIC_FEATURES[*]}"
    done

    cargo check --manifest-path "$REPO_DIR/electrsd/Cargo.toml" --no-default-features --features="$ELECTRSD_FEATURES"
}

for file in Cargo-minimal.lock Cargo-recent.lock; do
    cp --force "$file" Cargo.lock
    update_lock_files
    cp --force Cargo.lock "$file"
done
