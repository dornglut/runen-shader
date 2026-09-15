# Architecture

## System boundary

RunenShader is the standalone authority for reusable shader-source and
shader-compilation semantics.

Its canonical direction is:

```text
consumer / authoring host
    -> RunenShader semantic contracts
        -> selected frontend realization
            -> external compiler/tooling implementation
        -> exact canonical WGSL artifact + RunenShader evidence

consumer integration
    -> RunenGPU source admission / renderer / other downstream authority
```

The second path is composition, not a dependency inside the RunenShader semantic
core. RunenShader and RunenGPU are sibling frameworks.

## Semantic versus realization authority

The normative semantic model is under [spec/](spec/README.md). It owns logical
identity, immutable source revision meaning, explicit compilation input,
dependency resolution, frontend-profile boundaries, outcome classes, canonical
artifact formation, provenance, source mapping, and reproducibility.

Concrete compiler libraries, their private IR, reflection structures, filesystem
resolvers, caches, thread scheduling, and process state are implementation
realizations. They may implement accepted contracts but do not define them.

The public Rust crate now realizes the dependency-free semantic data boundary,
but no frontend compiler realization is accepted as implemented yet.

## Current repository shape

The current source tree contains:

- a public dependency-free semantic data kernel for explicit identities, exact
  source snapshots, closed exact-WGSL inputs/invocations, deterministic artifact
  evidence, identity source mapping, diagnostics, and outcome classes;
- no shader parser, compiler dependency, or compilation entry point yet;
- the normative `spec/` authority;
- root architecture, roadmap, status, testing, licensing, and agent entrypoints;
- a repository-local `xtask` that owns merge-readiness validation;
- a thin immutable CI caller.

Compiler realization code is admitted only by later issue-owned work that
preserves this public semantic boundary rather than exposing compiler-private
objects as authority.

## Implementation boundary

The root crate keeps implementation modules private and re-exports the accepted
public semantic types. Opaque identity representation, storage choices, and
module layout are implementation details unless `spec/` promotes a concrete
boundary.

Exact-WGSL artifact formation remains compiler-owned inside the crate: ordinary
callers can inspect `ShaderArtifact` but cannot construct success-shaped artifact
data directly. The later compiler realization will populate that type only after
successful accepted-profile validation.

## Documentation authority

| Concern | Canonical location |
| --- | --- |
| normative shader semantics | `spec/` |
| repository/system boundary | `ARCHITECTURE.md` |
| durable outcome sequence | `ROADMAP.md` |
| durable maturity/capability state | `STATUS.md` |
| merge-readiness and evidence | `TESTING.md` plus repository validator |
| executor rules | `AGENTS.md` |
| public landing and navigation | `README.md` |
| current licensing representation | `LICENSE` and `LICENSING.md` |
| implementation and tests | Rust source/tests when accepted |
| repository-local decision rationale | `docs/adr/` only when a real local ADR exists |
| dated durable verification reports | `docs/reports/` only when such evidence is accepted |
| live work state | GitHub issues, pull requests, and Projects |

Do not create empty documentation taxonomies for symmetry. Normative `spec/`
content must remain self-contained and may link only within `spec/`.

## Dependency rules

- The semantic core does not require RunenGPU, RunenRender, Runenwerk, or Runen
  language packages.
- A concrete frontend dependency is admitted only by accepted RunenShader work
  that proves its supported profile and conformance.
- Filesystem, registry, network, watcher, and hot-reload adapters are outside the
  semantic core unless later accepted work defines an explicit boundary.
- Downstream integrations map RunenShader products into downstream contracts;
  they do not share identity or mutate RunenShader authority.

## Failure boundary

Compilation failures are typed by the normative outcome model. Invalid input,
unsupported realization coverage, and operational tool failure remain distinct.
An internal RunenShader invariant violation is an implementation defect and must
fail closed rather than masquerade as an ordinary source error or publish an
artifact.
