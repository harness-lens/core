<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright © 2026 Cristian Camargo Filho -->

# harness-lens-core

The reference, provider-neutral Rust analysis engine for Harness Lens. It owns
source and finding models, normalized evidence scores, deterministic statistical
helpers, plugin contracts, report-sink contracts, and failure-isolated
orchestration.

Report schema version 1 includes one content-free record per source with UTF-8
bytes, Unicode characters, lines, inclusion depth, labeled token estimate,
configured static input cost, finding count, and safe provenance. Bounded
inclusion edges expose resolved, missing, cyclic, ignored, out-of-root, and
unavailable states.

Provider-neutral runtime records contain only tool/category identity, status,
duration, retry count, attributed cost, stable error class, model/asset/revision
identity, time window, method, assumptions, sample size, and uncertainty. Raw
arguments, output, transcripts, credentials, source, and stderr have no report
fields. Runtime defaults off and cannot alter deterministic findings or scores.
Effectiveness records require attributed revisions and comparison windows;
insufficient evidence remains an explicit state, never a fabricated file score.

Built-in plugins report deterministic adjacent repetition, conservative
same-intent instruction redundancy, opposite-modal instructions, provider
instruction-file conventions, and portable Agent Skills schema problems.
It also validates the documented structure of project-local Codex TOML and
command-rule assets.
Findings use UTF-8 byte spans so adapters can convert them without coupling the
core to LSP or an editor. The checked conventions and primary sources are
documented in [`docs/conventions.md`](docs/conventions.md).

This crate does not read files, call networks, execute agents, import model
providers, or know about Python and editors.

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
