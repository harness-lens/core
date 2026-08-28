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

BSD-3-Clause licensed.
