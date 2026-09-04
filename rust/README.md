<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright © 2026 Cristian Camargo Filho -->

# harness-lens-core

The reference, provider-neutral Rust analysis engine for Harness Lens. It owns
source and finding models, normalized evidence scores, deterministic statistical
helpers, plugin contracts, report-sink contracts, and failure-isolated
orchestration.

Built-in plugins report deterministic adjacent repetition and conservative,
explicitly heuristic opposite-modal instructions. Findings use UTF-8 byte spans
so adapters can convert them without coupling the core to LSP or an editor.

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
