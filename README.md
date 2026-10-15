# Eyleme examples

**One Eyleme brand — 15 October 2026:** [eyleme.dev](https://eyleme.dev/) is the shared home for Eyleme and its community. Eyleme remains a standalone, public Apache-2.0 project. **Eyleme Derleme** and **Eyleme Onama** follow a private-source, paid subscription model as optional products. Commercial services and terms are in development; pricing and availability are unpublished. This repository does not provide a subscription service.

The family is presented as **Derleme → Onama → Eyleme**: prepare data, authorize work, then execute. Eyleme keeps its own basic access controls and does not require either subscription for independent use. See the [public product-family overview](https://github.com/eyleme-dev/eyleme/blob/main/docs/en/product-family.md).

Technical identifiers retain their existing spelling for compatibility. Historical evidence preserves its original names and URLs.

Reproducible example applications. [Türkçe](README.tr.md).

**Status: one runnable external-consumer example.** `task-console/` is a small Rust client for the first-party core's local task service, pinned to core commit `b06b7bac1aa121a4fee81e9fd0f79e4cf2eeeb3f`. It does not contain the core runtime or any third-party crate.

## Scope

The task-console example creates and inspects a task, submits a deterministic step, and inspects its execution. Its session reader accepts only private files and an IPv4 loopback origin. The stock session is non-human, so a cancel request is expected to return `FORBIDDEN`; this example does not claim effective human-authorized stopping.

Run the Rust unit and documentation tests with `cargo test --workspace --locked`. Check formatting and warnings with `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`. To run the seven native acceptance scenarios, first build the exact pinned core CLI with its documented native command, then run `cargo run --locked --bin acceptance -- /path/to/eylema`. The acceptance runner checks the supplied checkout's `HEAD` before starting it and uses disposable private data directories. F6 covers an incompatible journal fixture; it is not a physical power-loss test.

Production Rust code, shared types, runtime/model execution and the CLI stay in [Eyleme](https://github.com/eyleme-dev/eyleme); product contracts live in [English docs](https://github.com/eyleme-dev/eyleme-docs-en) and [Turkish docs](https://github.com/eyleme-dev/eyleme-docs-tr). [The repository map](https://github.com/eyleme-dev/devguide/blob/main/docs/en/repository-map.md) defines the boundaries.

## Evidence boundary

The Rust profile and native acceptance runner establish this example's build and behavior on the stated host only. They do not establish GPU, model, training, cross-platform, or physical power-loss behavior. The pinned `eylema-contracts` package is the only external Cargo dependency allowed by the scoped examples profile.

Use [CONTRIBUTING](CONTRIBUTING.md) and [AGENTS](AGENTS.md). Keep private data, credentials, operational records and model weights outside Git. Public artifacts need their own approved storage, provenance and license decision; no store or service is provisioned here. First-party source and documentation are licensed under [Apache-2.0](LICENSE). Separately identified third-party material, model weights and datasets retain their own terms.
