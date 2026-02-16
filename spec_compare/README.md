# spec_compare — OpenRPC Spec vs corepc-types Comparison Tool

Compare the struct definitions generated from Bitcoin Core's OpenRPC
specification against the hand-maintained types in `types/src/`.

## Quick start

```bash
# Compare a single version
just spec-compare 30

# Compare all supported versions (v24–v30)
just spec-compare all
```

## What it does

For each version **N** the tool performs three steps:

1. **Codegen** — runs `corepc-codegen` against `specs/vN_*_openrpc.json`,
   producing a single `output/vN_generated.rs` containing every struct the
   spec implies.

2. **Inline** — walks `types/src/v17..vN`, inlines all `mod` declarations,
   and extracts every `pub struct` into `output/vN_inlined.rs`.

3. **Compare** — diffs the two files and writes a human-readable report to
   `output/vN_compare.txt`, which is also printed to the terminal.

## Directory layout

```
spec_compare/
├── README.md             ← you are here
├── run.sh                ← main entry point (called by `just spec-compare`)
├── compare_types.py      ← struct comparison logic
  ├── inline_corepc_version.py ← Rust module inliner / struct extractor
├── specs/                ← OpenRPC JSON files (one per Bitcoin Core release)
│   ├── v24_2_0_openrpc.json
│   ├── …
│   └── v30_2_0_openrpc.json
└── output/               ← generated artefacts (git-ignored)
    ├── vN_generated.rs
    ├── vN_inlined.rs
    └── vN_compare.txt
```

## Reading a comparison report

The report sections:

| Section                  | Meaning                                                                                                                                |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| **📊 Summary**           | High-level counts: matched, ignored, only-in-repo, only-in-spec.                                                                       |
| **🔵 Only in REPO**      | Repo has the struct but the spec does not. Should be **0** — add to `REPO_ONLY_IGNORE` or `TYPE_BRIDGE` if legitimate.                 |
| **🟡 Only in SPEC**      | Spec has the struct but the repo does not. Should be **0** — add to `SPEC_ONLY_IGNORE` or implement the type.                          |
| **🔄 Field differences** | Structs exist on both sides but fields differ. ⚠️ = significant (missing field or type mismatch), ℹ️ = minor (extra deprecated field). |
| **🔗 Bridge mappings**   | `TYPE_BRIDGE` entries that were actually used.                                                                                         |
| **⚠️ Ignored breakdown** | Why each ignored spec struct was suppressed.                                                                                           |

**The goal is `Only in repo: 0` and `Only in spec: 0` for every version.**

---

## Operations manual — adding a new Bitcoin Core version

When a new Bitcoin Core release (say **v31**) ships:

### 1. Obtain the OpenRPC spec

Generating the OpenRPC spec requires applying the git patch `core_spec_generation.patch` to Bitcoin Core target version that adds a new command called `bitcoin-cli getopenrpc`. After building core, it generates a file in the format `openrpc_<timestamp>.json` in Bitcoin Core root folder that can be copied over onto corepc folder as `spec_compare/specs/v<major>_<minor>_<patch>_openrpc.json`.

To apply the patch, copy `core_spec_generation.patch` over to Bitcoin Core clone in your target branch and run:

```bash
git apply --3way --index core_spec_generation.patch
```

To generate the OpenRPC spec, run:

```bash
just getopenrpc
```

This produces the `openrpc_<timestamp>.json` OpenRPC spec in Bitcoin Core root folder.

If you want to generate a new patch for Bitcoin Core including eventual porting fixes you had to make for it to work in a newer version, run:

```bash
git ls-files --others --exclude-standard -z | xargs -0 -r git add -N -- && git diff -u > core_spec_generation.patch
```

### 2. Run the comparison

```bash
just spec-compare 31
```

This will fail on the first run because:

- The codegen may produce new struct names not yet in `TYPE_BRIDGE`.
- The repo may not have types for new RPCs yet.

### 3. Triage the "only in spec" structs

For each struct listed under **🟡 Only in SPEC**:

| Situation                                                                                                             | Action                                                                                         |
| --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| The repo already has the type under a different name                                                                  | Add a `TYPE_BRIDGE` entry mapping spec name - repo name.                                       |
| The spec type is a nested sub-struct the repo handles with a shared type (e.g. `ScriptPubkey`, `RawTransactionInput`) | Add to `TYPE_BRIDGE` with `"IGNORE"` and a comment explaining which shared type the repo uses. |
| The spec type is a simple wrapper (returns bool/string/number)                                                        | Add to `SPEC_ONLY_IGNORE`.                                                                     |
| The type is genuinely new and needs implementing                                                                      | Implement it in `types/src/v31/`, then re-run.                                                 |
| The type only exists in older specs, not this version                                                                 | Add to `SPEC_ONLY_IGNORE_BY_VERSION` with the relevant version set.                            |

### 4. Triage the "only in repo" structs

For each struct under **🔵 Only in REPO**:

| Situation                                     | Action                                |
| --------------------------------------------- | ------------------------------------- |
| The type was removed/deprecated upstream      | Add to `REPO_ONLY_IGNORE`.            |
| The type exists for a subset of versions only | Add to `REPO_ONLY_IGNORE_BY_VERSION`. |
| The spec uses a different name                | Add a `TYPE_BRIDGE` entry.            |

### 5. Review field differences

Field diffs marked ⚠️ usually mean:

- A new field was added to the RPC response - add it to the repo struct.
- A field changed type - update the repo struct.

Field diffs marked ℹ️ are usually minor (extra deprecated field in repo).

### 6. Update the version loop

Edit `spec_compare/run.sh` and extend the `VERSIONS` array:

```bash
VERSIONS=(24 25 26 27 28 29 30 31)
```

### 7. Verify everything is clean

```bash
just spec-compare all
```

Confirm every version shows `Only in repo: 0` and `Only in spec: 0`.

### 8. Commit

```bash
git add spec_compare/
git commit -m "spec_compare: add v31 OpenRPC spec and mappings"
```

---

## Maintenance tips

- **`TYPE_BRIDGE`** is the authoritative mapping. Every spec struct must
  either match a repo struct by exact name, appear in `TYPE_BRIDGE`, or
  appear in `SPEC_ONLY_IGNORE`.

- **Comments matter.** Every `"IGNORE"` entry in `TYPE_BRIDGE` should have
  an inline `# comment` explaining _why_ it's ignored. The report
  categorises ignores by parsing these comments.

- **Version-specific sets** (`SPEC_ONLY_IGNORE_BY_VERSION`,
  `REPO_ONLY_IGNORE_BY_VERSION`) keep older version comparisons clean
  without polluting the global ignore lists.

- **No fuzzy matching.** If a struct is unmatched, it will show up in the
  report. This is intentional — silent mismatches are worse than noisy
  reports.
