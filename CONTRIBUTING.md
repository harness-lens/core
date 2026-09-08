> SPDX-License-Identifier: MPL-2.0
> Copyright © 2026 Cristian Camargo Filho

# How to contribute

Read the central [ecosystem contribution flow](https://github.com/harness-lens/harness-lens/blob/main/docs/architecture.md#how-to-contribute),
[architecture rules](https://github.com/harness-lens/harness-lens/blob/main/docs/architecture.md#architecture-rules),
and [LSP-visible rule path](https://github.com/harness-lens/harness-lens/blob/main/docs/architecture.md#adding-an-lsp-visible-rule).
Core owns report contracts, findings, scores, and rule behavior. Start rule work
with the [rule index](https://github.com/harness-lens/core/blob/main/docs/rules.md)
and [Core architecture](https://github.com/harness-lens/core/blob/main/docs/architecture.md).

Use Node.js 20 or newer and npm 11.

```bash
npm ci
npm test
npm run check
```

Keep validation deterministic. New rules need stable IDs, evidence, tests, and documentation. Do not let optional AI output affect findings or scores.

Also verify the Rust reference engine:

```bash
cd rust
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
cargo package --locked
```

## Licensing contributions

Contributions intentionally submitted to this repository are provided under
MPL-2.0. You must have the necessary rights to submit the work. When Covered
Software is distributed, modifications to MPL-covered core files remain
subject to the Source Code Form obligations in the license.
