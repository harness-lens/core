<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright © 2026 Cristian Camargo Filho -->

# Bounded lexical edit similarity

Harness Lens Native remains first-party MPL-2.0 software. This heuristic reports
lexical proximity only. It neither detects synonyms (`car` / `automobile`) nor
establishes intent, redundancy, correctness, or causation. No semantic score,
stemming, embedding, model call, or external executable is introduced.

## Adoption and sources

The edit model adopts unit-cost insertion, deletion, and substitution. Historical
source: V. I. Levenshtein, “Binary codes capable of correcting deletions,
insertions, and reversals,” Soviet Physics Doklady 10(8), 707–710 (1966),
[paper](https://nymity.ch/sybilhunting/pdf/Levenshtein1966a.pdf). Implementation is
original MPL-2.0 Rust; no paper or third-party algorithm implementation is copied.
Adjacent transposition is not a single edit in this implementation.

Normalize distance as `distance / max(left_length, right_length)` using normalized
Unicode scalar counts, not UTF-8 bytes or grapheme counts. No empty token enters
comparison. Default maximum distance is **0.2 inclusive**: `link` / `links` has
one edit over five scalars and qualifies. This is a conservative configurable
policy choice, not an empirically calibrated probability of redundancy.

Canonical composition follows [Unicode normalization, UAX #15](https://www.unicode.org/reports/tr15/).
NFC preserves compatibility distinctions that NFKC would remove. Apply NFC, full
non-Turkic case folding, then NFC again. Case folding can expand a scalar such
as `ß` to `ss`; [case-fold API](https://docs.rs/unicode-casefold/0.2.0/unicode_casefold/trait.UnicodeCaseFold.html).
This is not Unicode's `NFKC_Casefold` algorithm. Accents are not stripped.

Pinned library sources and licenses:

- [`unicode-normalization` 0.1.24](https://github.com/unicode-rs/unicode-normalization):
  MIT OR Apache-2.0; Unicode 16.0 normalization tables.
- [`unicode-properties` 0.1.3](https://github.com/unicode-rs/unicode-properties):
  MIT OR Apache-2.0; Unicode 16.0 general-category tables.
- [`unicode-casefold` 0.2.0](https://docs.rs/unicode-casefold/0.2.0/unicode_casefold/):
  library license and Unicode table version must be verified against the fetched
  pinned source before this change is accepted.
- [Unicode data license](https://www.unicode.org/license.txt).

## Boundaries and complexity

Tokenize runs beginning with a Unicode letter and continuing with letters or
combining marks. Original half-open UTF-8 spans survive normalization; normalized
tokens exist only in temporary memory. Number, punctuation, and underscore
boundaries are deliberately conservative. This is not linguistic segmentation;
long unsegmented scripts can be suppressed by the token length limit.

Default limits: minimum 4 scalars, maximum 64 original and normalized scalars,
1,024 token occurrences (including short tokens), 10,000 candidate pairs,
256 KiB total input, and 1,024 sources. Hard configuration ceilings: 256 scalars,
4,096 tokens, 100,000 pairs, 4 MiB, and 4,096 sources. Zero budgets and invalid or
non-finite thresholds yield an explicit invalid-configuration state.

Sources sort by local path before tokenization; duplicate paths are rejected.
Candidates follow source path and span order. Every visited pair consumes budget,
including exact matches and length-rejected pairs, so exclusions cannot evade
the work bound. Exceeding an input, token, or pair bound produces stable reasons
both on the report and every emitted finding. A token cut by the input bound is
never analyzed as a complete token. Overlong tokens are skipped, never truncated
into misleading candidates.

Length difference rejects impossible candidates immediately. Two-row banded
dynamic programming exits once every reachable row cell exceeds the allowed
integer edit budget. Memory is O(L) per distance computation plus O(TL) temporary
tokens and bounded findings; worst-case comparison work is O(P L²), where P and L
have explicit hard ceilings. Row clearing remains O(L), even for a narrow band.
NFC buffering is bounded by the original maximum token length.

## Assumptions, rejection, and false positives

Normalized exact equality is excluded from fuzzy findings and remains the domain
of exact-match rules. Case-only and canonically equivalent spellings therefore
produce no fuzzy duplicate. Exact-match rules retain their own documented scope;
excluding a lexical pair does not promise a separate exact finding for that pair.

Similar spelling can occur in unrelated words; minimum length and distance limits
reduce noise but cannot eliminate it. Token pairs across distant instructions may
have unrelated purposes. Full case folding is locale-independent and may not
match local language expectations. Grapheme-aware distance, language-specific
segmentation, stemming, synonym dictionaries, embeddings, and semantic claims
are deliberately excluded. Review local spans before acting on a candidate.

Output contains fixed assumptions, method `heuristic`, two spans, normalized
distance, configured threshold, normalization identifier, and completeness.
It contains no token strings or source excerpts. Findings remain under `lexical`,
outside deterministic findings, scores, and quality aggregation.
