# RunenShader Agent Guide

Start with `README.md`, `ARCHITECTURE.md`, `TESTING.md`, `ROADMAP.md`,
`STATUS.md`, and `spec/README.md`. For nontrivial work, read the current owning
issue and the accepted Dornglut Engineering authority before editing.

## Scope

RunenShader owns reusable shader-source and shader-compilation semantics. The
normative product contract lives under `spec/`.

RunenShader does not own Runen language semantics, RunenGPU GPU/program
admission or execution semantics, renderer image-formation policy, application
recovery, filesystem watching/hot-reload activation, or package-registry and
network-fetch policy.

## Ownership rules

- Keep one semantic authority per concern.
- Normative shader semantics live only under `spec/`; architecture, roadmap,
  status, tests, implementation, and issues may reference them but do not create
  competing normative rules.
- Logical identities must not be inferred from filesystem paths, hashes,
  compiler-private identities, cache entries, or RunenGPU identities.
- Frontend language semantics remain owned by the selected external language
  specification. A RunenShader frontend profile records supported semantics; it
  does not redefine the language.
- Compiler IR, reflection, resolver internals, caches, and process state are
  realization details unless an accepted RunenShader contract explicitly
  promotes a boundary.
- RunenShader canonical WGSL is a product. It is not RunenGPU program-interface,
  binding, pipeline, or execution authority.
- Do not introduce a generic plugin surface, compatibility shim, forwarding
  package, duplicate source authority, or hidden fallback to ambient state
  without accepted issue-owned work.

## Required workflow

1. Re-resolve accepted `main`, current issues, and relevant accepted Engineering
   authority.
2. Select one owning issue with decision-complete scope before substantive work.
3. Read every modified existing file from the exact publication parent.
4. Build one complete isolated candidate from the accepted base. Do not expose
   sequential partial repository states.
5. Review the complete accepted-base-to-candidate diff against issue scope.
6. Publish the complete candidate on an isolated branch and open a draft pull
   request.
7. Run the repository-owned `cargo validate` against the exact feature head.
   Hosted exact-head CI is required when local Rust execution is unavailable.
8. Any feature-head movement invalidates earlier validation and review evidence.
9. Reconcile the final exact head with current `main`, review state, and
   repository authority before merge. Merge only the exact reviewed SHA.
10. Record accepted delivery evidence in the owning issue after merge.

## Validation

The canonical command is:

```text
cargo validate
```

Focused checks may assist development but never replace that baseline.
