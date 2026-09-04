> SPDX-License-Identifier: MPL-2.0
> Copyright © 2026 Cristian Camargo Filho

# @harness-lens/core

Provider-neutral contracts and deterministic analysis for Harness Lens. The
repository contains the reference Rust engine and retains the TypeScript package
for existing npm consumers.

| Implementation | Role | Package |
| --- | --- | --- |
| [`rust/`](rust/) | Reference domain and analysis engine | `harness-lens-core` |
| [`src/`](src/) | TypeScript compatibility implementation | `@harness-lens/core` |

Recognized inputs:

- `AGENTS.md`
- `CLAUDE.md`
- `GEMINI.md`
- `.github/copilot-instructions.md`
- `.cursor/rules/*`

```ts
import { scanRepository } from "@harness-lens/core";

const report = await scanRepository(process.cwd(), {
  profile: "coding-agent/v1",
});
```

Coverage is evaluated only when an explicit profile is selected. Alignment and cost remain `not-evaluated` until callers supply a concrete reference. Optional AI interpreters consume the completed report and cannot change deterministic metrics.

## Pipeline

```text
discover → load → normalize → validate → measure → snapshot → compare → render
```

This `0.0.x` API is intentionally small and may change before `1.0.0`.

## Ecosystem

Core owns pure domain behavior. Integration surfaces live in separate
repositories and depend inward on these contracts:

- [SDK](https://github.com/harness-lens/sdk) — embedding, configuration, Python, and discovery adapters
- [CLI](https://github.com/harness-lens/cli) — terminal interface
- [Language Server](https://github.com/harness-lens/language-server) — editor diagnostics
- [VS Code](https://github.com/harness-lens/harness-lens-vscode) — editor presentation
- [Harness Lens](https://github.com/harness-lens/harness-lens) — ecosystem documentation and pinned repository composition

## Development

```bash
npm install
npm test
npm run check
npm pack

cd rust
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## License

Early namespace-reservation versions used BSD-3-Clause. The official functional
implementation is licensed under MPL-2.0. Harness Lens Core is the shared
deterministic core of the ecosystem; when Covered Software is distributed,
modified MPL-covered core files must remain available in Source Code Form under
the license. See [LICENSING](LICENSING.md), [COPYRIGHT](COPYRIGHT), and
[TRADEMARKS](TRADEMARKS).
