<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright © 2026 Cristian Camargo Filho -->

# harness-lens-core

The reference, provider-neutral Rust analysis engine for Harness Lens. It owns
source and finding models, normalized evidence scores, deterministic statistical
helpers, plugin contracts, report-sink contracts, and failure-isolated
orchestration.

Built-in plugins report deterministic adjacent repetition, exact duplicate
lines/paragraphs with related source locations, conservative opposite-modal
instructions, and source-size/token-cost evaluation. Findings use UTF-8 byte
spans so adapters can convert them without coupling the core to LSP or an
editor. Monetary cost is calculated only when a caller supplies a price and is
explicitly modeled as input injection cost per invocation.

This crate does not read files, parse TOML, call networks, execute agents, import
model providers, or know about Python and editors.

## Development

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Ecosystem

- [Rust/Python SDK](https://github.com/harness-lens/sdk)
- [CLI](https://github.com/harness-lens/cli)
- [Language server](https://github.com/harness-lens/language-server)
- [VS Code extension](https://github.com/harness-lens/harness-lens-vscode)
- [Project hub](https://github.com/harness-lens/harness-lens)

## License

MPL-2.0. See the repository [LICENSE](../LICENSE), [LICENSING](../LICENSING.md),
[COPYRIGHT](../COPYRIGHT), and [TRADEMARKS](../TRADEMARKS).
