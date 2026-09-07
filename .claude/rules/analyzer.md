---
paths:
  - "crates/tyrus_analyzer/**"
  - "crates/tyrus_diagnostics/**"
---

# Analyzer & Diagnostics Rules (binding — R14, envelope contract)

- **JSON envelope is a public contract** (`report.rs`): `schemaVersion` is 1. Adding
  fields consumers can ignore = allowed without bump. Renaming/removing/retyping a
  field or changing `status` semantics = breaking → bump the version and document in
  the PR (release-plz owns CHANGELOG). `status` reflects hard errors only; `--strict`
  affects exit code, never `status`.
- **R14:** every `TyrusError` variant carries a unique stable code (`TYRUS-EXXXX`) and
  an `ErrorCategory`. New variant = new code in the correct block (E0 parse, E1 analyze,
  E2 codegen, E3 io, E4 format) + arm in `stable_code()`/`category()` (compiler enforces
  exhaustiveness) + instance in the `all_variants()` test list. Codes are never reused
  or renumbered. Tests assert codes, never message text.
- Hard errors (lint violations: `var`, `any`, `eval`, for-in, delete, with, labeled
  statements, ambiguous main) are `TyrusError`. Soft findings (blocked APIs like
  `setTimeout`, `document`) are `Diagnostic` with severity — they only become fatal
  under `--strict`.
- **Unknown decorators are not findings** (owner rule, #265): `DecoratorVisitor` consults
  `DecoratorKind::from_name` only to build the DI graph. A decorator outside the registry
  is valid TypeScript that codegen translates generically — never a `TyrusError`, and a
  `Diagnostic` only under `--strict`, advisory. The analyzer rejects language-level
  violations (lint codes), not names it has not seen.
- Spans: `LintVisitor` subtracts 1 from SWC's `span.lo` offset. Keep new visitors
  consistent with that convention (divergence here already produced off-by-one labels).
- `tyrus_diagnostics` and `tyrus_common`/`tyrus_decorator_kinds` are boundary crates:
  `#![deny(missing_docs)]` — every new public item needs a doc comment; doc-tests are
  the preferred spec for pure functions.
