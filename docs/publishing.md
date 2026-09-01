> SPDX-License-Identifier: MPL-2.0
> Copyright © 2026 Cristian Camargo Filho

# Publishing

The first `@harness-lens/core` release must be published interactively by an npm organization owner. After the package exists, configure npm trusted publishing for `.github/workflows/publish.yml` and the `npm` GitHub environment.

```bash
npm login
npm publish --access public --provenance
```

Later releases are created from GitHub Releases. Never store npm tokens in the repository.
