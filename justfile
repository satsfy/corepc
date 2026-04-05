set export

REPO_DIR := `git rev-parse --show-toplevel`
NIGHTLY := `cargo metadata --no-deps --format-version 1 | jq -re '.metadata.rbmt.toolchains.nightly // .workspace_metadata.rbmt.toolchains.nightly'`

# Targets where `--all-features` is used.
ALL_FEATURE_CRATES := "bitreq client fuzz jsonrpc types verify"

# Targets with conflicting features and only `SPECIFIC_FEATURES` are used.
SPECIFIC_FEATURES_CRATES := "integration_test node"
SPECIFIC_FEATURES := "latest"

alias ulf := update-lock-files
alias l := lint
alias li := lint-integration-tests
alias lv := lint-verify

default:
  @just --list

# Cargo build everything.
build:
  for crate in {{ALL_FEATURE_CRATES}}; do cargo build --manifest-path "$REPO_DIR/$crate/Cargo.toml" --all-targets --all-features; done

  for crate in {{SPECIFIC_FEATURES_CRATES}}; do cargo build --manifest-path "$REPO_DIR/$crate/Cargo.toml" --all-targets --no-default-features --features={{SPECIFIC_FEATURES}}; done

# Cargo check everything.
check:
  for crate in {{ALL_FEATURE_CRATES}}; do cargo check --manifest-path "$REPO_DIR/$crate/Cargo.toml" --all-targets --all-features; done

  for crate in {{SPECIFIC_FEATURES_CRATES}}; do cargo check --manifest-path "$REPO_DIR/$crate/Cargo.toml" --all-targets --no-default-features --features={{SPECIFIC_FEATURES}}; done

# Lint everything.
lint: lint-verify lint-integration-tests
  for crate in {{ALL_FEATURE_CRATES}}; do cargo +$NIGHTLY clippy --manifest-path "$REPO_DIR/$crate/Cargo.toml" --all-targets --all-features -- --deny warnings; done

  for crate in {{SPECIFIC_FEATURES_CRATES}}; do cargo +$NIGHTLY clippy --manifest-path "$REPO_DIR/$crate/Cargo.toml" --all-targets --no-default-features --features={{SPECIFIC_FEATURES}} -- --deny warnings; done

lint-verify:
  $REPO_DIR/contrib/lint-verify.sh

lint-integration-tests:
  $REPO_DIR/contrib/lint-integration-tests.sh

# Run cargo fmt
fmt:
  cargo +$NIGHTLY fmt --all
  cargo +$NIGHTLY fmt --manifest-path $REPO_DIR/integration_test/Cargo.toml
  cargo +$NIGHTLY fmt --manifest-path $REPO_DIR/verify/Cargo.toml

# Check the formatting
format:
  cargo +$NIGHTLY fmt --all --check

# Generate documentation.
docsrs *flags:
  for crate in {{ALL_FEATURE_CRATES}}; do RUSTDOCFLAGS="--cfg docsrs -D warnings -D rustdoc::broken-intra-doc-links" cargo +$NIGHTLY doc --manifest-path "$REPO_DIR/$crate/Cargo.toml" --all-features {{flags}}; done

  for crate in {{SPECIFIC_FEATURES_CRATES}}; do RUSTDOCFLAGS="--cfg docsrs -D warnings -D rustdoc::broken-intra-doc-links" cargo +$NIGHTLY doc --manifest-path "$REPO_DIR/$crate/Cargo.toml" --no-default-features --features={{SPECIFIC_FEATURES}} {{flags}}; done

# Update the recent and minimal lock files.
update-lock-files:
  contrib/update-lock-files.sh
