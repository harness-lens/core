<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright © 2026 Cristian Camargo Filho -->

# Phase 1 Core draft verification

This is the Core portion of Phase 1, based on foundation commit
`b9628198f34d685a5ebad78a614e17de277079ab`. It adds bounded lexical evidence and
provider metadata/report/merge contracts. SDK adapters, Language Server provider
transport, and editor services are separate pending owning-repository work.
This draft is not ready to merge or use as an accepted downstream dependency.

## Local evidence — 2026-09-06

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

## Reviewer verification

On a development machine authorized to fetch dependencies, regenerate and review
the lockfile before running locked checks. `rust/Cargo.lock` currently predates
the three pinned Unicode dependencies in `rust/Cargo.toml`; the Rust 1.85 CI job
uses `--locked` and requires that lockfile update. Do not hand-author checksums
or weaken the CI lockfile check.

From `rust/`, with stable and Rust 1.85.1 available:

```bash
cargo generate-lockfile
git diff -- Cargo.lock
cargo fmt --all --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
cargo +1.85.1 test --all-features --locked
cargo package --locked --allow-dirty
```

`--allow-dirty` permits package verification while the generated lockfile is
under review. Commit the reviewed lockfile and any required fixes, then repeat
`cargo package --locked` on the clean commit. Packaging does not publish.

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
- Resolve dependency/lockfile issues, compile the complete crate, and fix any
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
