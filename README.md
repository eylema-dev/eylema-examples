# eylema-examples

Reproducible example applications. [Türkçe](README.tr.md).

**Status: repository scaffold only.** No application, benchmark, catalog entry, trained model or training run is delivered by this setup.

## Scope

First task-console example: create, run, stop, save and reopen against a pinned core release. Add an executable example only after the core and its API exist.

Production Rust code, shared types, runtime/model execution and the CLI stay in [eylema](https://github.com/eylema-dev/eylema); product contracts live in [English docs](https://github.com/eylema-dev/eylema-docs-en) and [Turkish docs](https://github.com/eylema-dev/eylema-docs-tr). [The repository map](https://github.com/eylema-dev/devguide/blob/main/docs/en/repository-map.md) defines the boundaries.

## First acceptance gate

Before implementation, open a scoped task with input/output contracts, an exact core revision, meaningful failure cases, dependency inventory and named execution targets. Replace the documentation-only source guard with suitable tests in the same PR that introduces implementation. Documentation CI is not runtime or hardware evidence.

Use [CONTRIBUTING](CONTRIBUTING.md) and [AGENTS](AGENTS.md). Keep private data, credentials, operational records and model weights outside Git. Public artifacts need their own approved storage, provenance and license decision; no store or service is provisioned here. Distribution license selection is pending.
