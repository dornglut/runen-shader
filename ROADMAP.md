# Roadmap

This roadmap records durable RunenShader outcome sequencing. GitHub issues and
pull requests own live delivery state; this file does not duplicate their
inventories, exact heads, or temporary blockers.

## R0 — Semantic kernel

Establish one repository-owned shader-source/toolchain semantic authority and
realize it with the first independently usable deterministic path from explicit
source input to exact canonical WGSL.

Exit properties:

- normalized source/package/module identity and immutable revision semantics;
- closed deterministic compilation inputs and dependency resolution;
- explicit frontend-profile versus compiler-realization contracts;
- typed outcomes, diagnostics, provenance, source mapping, and reproducibility;
- one first concrete frontend path accepted through its own evidence;
- canonical WGSL artifact formation without acquiring RunenGPU execution
  authority;
- repository-owned validation and conformance for the accepted public surface.

## R1 — Composition breadth

Expand supported source composition only from demonstrated consumer needs and
frontend evidence. Additional frontend profiles or language features are
accepted independently; no universal frontend plugin model is presumed.

The outcome is multiple useful authoring/composition paths that preserve the R0
identity, resolution, artifact, and reproducibility invariants.

## R2 — Downstream conformance

Prove stable composition with real independent consumers while preserving sibling
authorities. Downstream integrations must explicitly map RunenShader artifacts
into their own contracts rather than sharing implementation identities.

This stage may include renderer, compute, or other consumers, but it does not
transfer renderer, GPU, application, or language semantics into RunenShader.

## Evolution rule

A roadmap stage authorizes no implementation by itself. Each substantive change
requires decision-complete repository-owned work, exact-head validation, and
accepted evidence. Stages may be split, reordered, simplified, or deleted when
current evidence changes the cheapest correct path.
