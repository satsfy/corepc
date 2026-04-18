# Bitcoin Core test fixtures

This directory contains a vendored copy of Bitcoin Core functional tests and
test data.

The goal is not to run Bitcoin Core's test suite here. The goal is to mine
Bitcoin Core's own RPC tests for real response shapes, then use those responses
as fixtures for `corepc` deserialization tests.

Why this is useful:

- exercises more than one hand-written happy path per RPC method
- catches optional fields that are missing, present, or version-dependent
- catches fields that appear to work only because serde filled in defaults
- keeps `corepc` closer to Bitcoin Core's actual RPC behavior

Useful places to look:

- `functional/rpc_*.py` for RPC-focused functional tests
- `functional/wallet_*.py` for wallet RPC behavior
- `functional/data/*.json` for existing static test vectors
- `get_previous_releases.py` for downloading older Bitcoin Core binaries when
  comparing behavior across versions

When adding coverage from this directory, prefer small focused fixtures. Capture
the RPC response JSON, deserialize it into the matching `corepc` type, and assert
on fields that matter for the bug or RPC method being covered. Do not copy large
Bitcoin Core test flows into Rust unless the flow itself is needed.

This directory is source material. The actual `corepc` tests should live with the
Rust crate that owns the type or client method being tested.
