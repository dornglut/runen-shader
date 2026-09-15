# RunenShader normative specification

This directory is the sole normative authority for RunenShader shader-source and
shader-compilation semantics. Repository architecture, roadmap, status, issues,
implementation, tests, and downstream integrations may explain or realize this
contract but do not create competing semantic rules.

## Requirement language

The keywords `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` express
normative strength in this specification:

- `MUST` / `MUST NOT`: required for conformance;
- `SHOULD` / `SHOULD NOT`: expected unless a stronger accepted reason is
  documented and conformance remains intact;
- `MAY`: permitted but not required.

Requirement identifiers are stable semantic references within this repository.
Changing an accepted requirement requires ordinary reviewed specification work;
implementation convenience alone does not change its meaning.

## Decision states

Normative sections use these states where a decision is not yet fully fixed:

- **Accepted** — part of the current RunenShader semantic contract.
- **Open** — intentionally unresolved; no implementation choice is authorized by
  the absence of a decision.
- **Deferred** — explicitly outside the current contract and requires later
  accepted work before becoming supported.

An `Open` or `Deferred` item is not a generic extension point and does not grant
implementations freedom to publish incompatible public semantics.

## Specification map

- [Semantic model](semantic-model.md) — canonical identity, source snapshot,
  compilation, resolution, artifact, diagnostics, provenance, reproducibility,
  outcome, cache, and downstream-boundary semantics.
- [Exact WGSL frontend profile](frontend-wgsl.md) — first concrete frontend
  profile, exact-artifact rule, validation realization contract, source mapping,
  outcome specialization, and conformance obligations.

Normative files may link only to other files inside `spec/`. External language
specifications and compiler documentation are explicit inputs to concrete
frontend profiles; they do not silently modify this repository's normative
contract.
