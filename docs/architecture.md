> SPDX-License-Identifier: MPL-2.0
> Copyright © 2026 Cristian Camargo Filho

# Architecture

The Rust `harness-lens-core` crate is the reference engine. The
`@harness-lens/core` package remains a TypeScript compatibility implementation.
Both own domain behavior and structured report contracts; neither owns terminal,
editor, transport, persistence, agent-framework, or AI-provider behavior.

```text
discovery → safe loading → Markdown normalization → deterministic validation
          → metrics → report → snapshot/compare
```

Rules emit evidence-bearing findings. Metrics that require a reference use `not-evaluated` when none is provided. Structural quality is not presented as proof of behavioral effectiveness.

Adapters belong elsewhere:

- `@harness-lens/cli`: terminal and future TUI
- `@harness-lens/sdk`: embedding facade
- `@harness-lens/language-server`: editor diagnostics
- `@harness-lens/vscode`: VS Code UX

Behavioral probes may later compare declared rules with observed agent executions. They remain separate from static validation.

## Dependency direction

```text
core <- sdk/adapters <- cli
                    <- language-server <- VS Code
```

Downstream Rust repositories pin this repository by immutable Git revision until
the crates are published. The umbrella repository pins each component as a Git
submodule, making a reproducible ecosystem revision without restoring monorepo
coupling.
