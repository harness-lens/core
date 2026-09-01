> SPDX-License-Identifier: MPL-2.0
> Copyright © 2026 Cristian Camargo Filho

# Architecture

`@harness-lens/core` owns domain behavior and structured report contracts. It has no terminal, editor, transport, persistence, or AI-provider dependency.

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
