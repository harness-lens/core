> SPDX-License-Identifier: MPL-2.0
> Copyright © 2026 Cristian Camargo Filho

# @harness-lens/core

Deterministic core for Harness Lens. It discovers, safely loads, normalizes, validates, measures, snapshots, and compares AI-agent harness files.

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

## Development

```bash
npm install
npm test
npm run check
npm pack
```

## License

Early namespace-reservation versions used BSD-3-Clause. The official functional
implementation is licensed under MPL-2.0. Harness Lens Core is the shared
deterministic core of the ecosystem; when Covered Software is distributed,
modified MPL-covered core files must remain available in Source Code Form under
the license. See [LICENSING](LICENSING.md), [COPYRIGHT](COPYRIGHT), and
[TRADEMARKS](TRADEMARKS).
