# Eyleme examples

**Name update — 14 October 2026:** The product is **Eyleme** (formerly Eylema); the companion product is **Derleme** (formerly Derlema), and **Onama** keeps its name. The GitHub organization is now `eyleme-dev` (formerly `eylema-dev`). The main repository is now `eyleme-dev/eyleme`. On 15 October 2026 the six `eylema-*` child repositories became `eyleme-*`. Domains, commands, crate/package names, environment variables, data paths and protocol/schema identifiers retain their current spelling for compatibility. Historical evidence preserves its original names and URLs.

Reproducible example applications. [Türkçe](README.tr.md).

**Status: one runnable external-consumer example.** `task-console/` is a small Rust client for the first-party core's local task service, pinned to core commit `b06b7bac1aa121a4fee81e9fd0f79e4cf2eeeb3f`. It does not contain the core runtime or any third-party crate.

## Scope

The task-console example creates and inspects a task, submits a deterministic step, and inspects its execution. Its session reader accepts only private files and an IPv4 loopback origin. The stock session is non-human, so a cancel request is expected to return `FORBIDDEN`; this example does not claim effective human-authorized stopping.

Run the Rust unit and documentation tests with `cargo test --workspace --locked`. Check formatting and warnings with `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`. To run the seven native acceptance scenarios, first build the exact pinned core CLI with its documented native command, then run `cargo run --locked --bin acceptance -- /path/to/eylema`. The acceptance runner checks the supplied checkout's `HEAD` before starting it and uses disposable private data directories. F6 covers an incompatible journal fixture; it is not a physical power-loss test.

Production Rust code, shared types, runtime/model execution and the CLI stay in [Eyleme](https://github.com/eyleme-dev/eyleme); product contracts live in [English docs](https://github.com/eyleme-dev/eyleme-docs-en) and [Turkish docs](https://github.com/eyleme-dev/eyleme-docs-tr). [The repository map](https://github.com/eyleme-dev/devguide/blob/main/docs/en/repository-map.md) defines the boundaries.

## Evidence boundary

The Rust profile and native acceptance runner establish this example's build and behavior on the stated host only. They do not establish GPU, model, training, cross-platform, or physical power-loss behavior. The pinned `eylema-contracts` package is the only external Cargo dependency allowed by the scoped examples profile.

Use [CONTRIBUTING](CONTRIBUTING.md) and [AGENTS](AGENTS.md). Keep private data, credentials, operational records and model weights outside Git. Public artifacts need their own approved storage, provenance and license decision; no store or service is provisioned here. First-party source and documentation are licensed under [Apache-2.0](LICENSE). Separately identified third-party material, model weights and datasets retain their own terms.
