<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright © 2026 Cristian Camargo Filho -->

# Provider contracts, version 1

Draft acceptance status and reviewer commands are recorded in
[Phase 1 Core verification](phase1-verification.md).

`harness-lens-native` identifies **Harness Lens Native**, always available,
always selected for deterministic analysis, and MPL-2.0. `codeburn` identifies
optional external CodeBurn. Future providers require an explicit local adapter;
remote metadata never registers executable behavior.

Core's `providers` module defines metadata, capabilities, supported operating
systems, required settings without values, availability, installation status,
safe error classes, declarative installation previews, health, refresh generations,
provider report envelopes, and merge provenance. SDK owns detection and
installation boundaries. Protocol and editor adapters own transport and consent.

Installation plans are local serializable previews without deserialization.
They declare package source, requested version when known, command words,
expected executable, license URL, network and filesystem effects, and restart
requirements. A preview is never executable authority. Compiled-in adapters must
reconstruct allowlisted process arguments and recheck consent and trust before
launch. Native has no installation or removal action.

`AggregateEnvelope` supplements the existing native report. It cannot mutate
that report. `reports[provider_id]` namespaces measurements; metrics retain raw
values and are never implicitly averaged or combined into native scores.
Contributions use closed metric and assumption enums, sample size, original
UTF-8 evidence offsets into the native source table, and optional fixed-byte
fingerprints. No raw JSON, message, command argument, transcript, output, source
string, or credential field exists in this mapping contract.

Only a matching explicit fingerprint **and identical entire contribution** permit
deduplication. All contributing IDs remain in provenance. Equal fingerprints
with conflicting values, methods, evidence, or assumptions remain side-by-side.
Without a fingerprint, even equal values remain independent contributions.

Catalog IDs come from trusted local registration. Merge rejects unknown IDs,
duplicate provider attempts, more than 16 providers, more than 2,048 contributions
per provider, non-finite values, reversed spans, and oversized evidence/assumption
lists. Invalid individual reports become safe per-provider failures. Statistical
contributions require sample size; probabilistic contributions are reserved until
the prior and interval contract is provided. Non-deterministic contributions
require explicit assumptions.

Hosts assign monotonically increasing generations when refreshes are requested.
Late older completions cannot replace newer accepted state. Catalog and report
output are sorted independently of provider completion order. Native remains in
selection even if callers omit it. Runtime failures may retain a validated
previous snapshot with `stale` health. Hosts must disable retention on workspace,
trust, selection, or mode changes; blocked or invalid configuration clears the
affected snapshot. No error can disable Native analysis.

Phase 2 presentation work remains: provider selection interface, installation
confirmation presentation, health and stale-state display, side-by-side conflicts,
lexical evidence navigation, accessibility, and final panel layout/styling. These
contracts prescribe no visual layout.
