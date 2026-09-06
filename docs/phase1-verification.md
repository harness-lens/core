<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright © 2026 Cristian Camargo Filho -->

# Phase 1 Core draft verification

This is the Core portion of Phase 1, based on foundation commit
`b9628198f34d685a5ebad78a614e17de277079ab`. It adds bounded lexical evidence and
provider metadata/report/merge contracts. SDK adapters, Language Server provider
transport, and editor services are separate pending owning-repository work.
This draft is not ready to merge or use as an accepted downstream dependency.

## Initial local evidence — 2026-09-06

Node 24.18.0, npm 11.16.0, installed Rust stable 1.97.1.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` in `rust/` | Passed |
| `git diff --check` | Passed |
| `npm test` | Passed, one test file |
| `npm run check` | Passed |
| `npm pack --dry-run` with temporary writable cache | Passed, 67 package files |
| Standalone bounded-distance/reference test | Passed, one extracted Rust test |
| `cargo test --all-features --offline` | Blocked: missing `unicode-casefold` |
| `cargo clippy --all-targets --all-features --offline -- -D warnings` | Same dependency blocker |
| `cargo package --locked --offline --allow-dirty` | Same dependency blocker |
| Rust 1.85.1 | Not installed; no local MSRV verification |

TypeScript checks reused existing dependencies through a temporary `node_modules`
symlink, removed afterward. No `npm ci`, dependency download, toolchain install,
or external report provider install was performed. The npm dry-run checks the
existing TypeScript package; it does not validate the new Rust contracts.

The standalone test compiled the actual `bounded_distance` function and its
updated test with `rustc --edition=2024 --test`. It compares against an
independent full matrix over every binary string of length zero through five
at budgets zero through five. It does not verify Unicode dependencies, the
full crate, serialization, or provider merging.

## Lockfile correction

The initial PR omitted the three pinned Unicode dependencies from the committed
lockfile. MSRV testing rejected the outdated lockfile; the stable build generated
a replacement, then packaging rejected the resulting dirty worktree.

Cargo regenerated `rust/Cargo.lock` from registry metadata, including the pinned
Unicode crates and their `tinyvec` dependencies. Existing dependency versions
are preserved. This fetched registry metadata only; it did not download crate
source, compile dependencies, or install toolchains or external providers.
Stable Clippy and tests now also use `--locked`, so all dependency-resolving CI
checks enforce the committed graph. Packaging retains its clean-worktree check.

## Reviewer verification

Use a clean checkout of the PR with the committed lockfile. No lockfile
regeneration is required for normal verification.

From `rust/`, with stable and Rust 1.85.1 available:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
cargo +1.85.1 test --all-features --locked
cargo package --locked
git diff --exit-code -- Cargo.lock
```

Packaging does not publish. Keep `--locked` and the clean-worktree requirement
enabled so local checks match CI.

From the repository root:

```bash
npm ci
npm test
npm run check
npm pack --dry-run
git diff --check
```

## Acceptance work remaining

- Verify the pinned case-fold library license and Unicode table version against
  its source; complete the pending entry in
  [the lexical prior-art note](prior-art/lexical-similarity.md).
- Verify the complete crate against the committed lockfile, and fix any
  compiler, Clippy, test, packaging, or MSRV failures before acceptance.
- Review provider bounds, content-free mapping, fingerprint conflicts, and stale
  snapshot rules. Probabilistic contributions are explicitly rejected until a
  prior/interval contract is defined; they are not implemented by this draft.
- Account for new public `HarnessLensConfig.lexical` and `AnalysisReport.lexical`
  fields in downstream Rust struct literals. Serde defaults support older wire
  reports/configuration; that does not make Rust struct literals source compatible.
- Complete and independently verify SDK services, Language Server transport,
  and editor service/consent boundaries with immutable upstream pins.
- Merge only with authorization. Phase 2 UI and hub composition updates remain
  gated on accepted Phase 1 owning-repository merges and pins.
