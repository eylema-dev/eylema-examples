# G17 task revision 2 — pinned external consumer

## Status

Execution revision 2 is in progress under the maintainer's direct direction to complete all Goals in action-plan order, with each completed Goal merged after current CI and separate review. Goal G17, issue [eylema-examples #3](https://github.com/eylema-dev/eylema-examples/issues/3).

## Baselines and dependency

- Examples `main` base: `6f013b265e7b457198c8fb821e457bc856d9bb0a`.
- Accepted G16 core dependency: `eylema-dev/eylema` main `b06b7bac1aa121a4fee81e9fd0f79e4cf2eeeb3f` (G16 core PR #49; tree `42cc29d9ad1155bd1ef42c48d3a0d49aeb5368dd`). The acceptance runner checks commit, tree and clean worktree, then builds the CLI from that checkout before exercising it. No release or tag is claimed; the consumer pins this immutable commit.
- Accepted G16 guide integration: `eylema-dev/devguide` main `f7f5b2c22e30ac6dcf41f16c012443a7fe95cd8d` (G16 PR #23, main CI `38229783239`).
- Product contract bindings: architecture 0.3.1, API 0.1.0, development protocol 0.1.0, and F1–F7 in `eylema-docs-en/docs/project-structure.md` at its current-main revision `4e8f91ce6515bb7d0a02aab89c97cfca8e086a4f`.

## Proposed scope

Implement one small, first-party Rust consumer under `task-console/` that compiles against `eylema-contracts` from the exact core Git commit above and talks only to the local loopback task service. It must not copy core implementation code or introduce a third-party Cargo dependency. Keep English canonical and update the paired Turkish README and reviewed fingerprints.

G17 requires a narrowly linked devguide profile-gate change: let `eylema-examples` enter the existing `examples` Rust profile and permit only the explicitly pinned first-party core contract dependency. Add negative tests proving all other external dependencies and mutable/unpinned core revisions still fail. Update required shared-tool copies and the immutable repository-set pin after the final member commits. This is part of the G17 consumer acceptance, not a general dependency-policy relaxation.

Expected examples-repository paths: `task-console/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `repository.json`, `README.md`, `README.tr.md`, `tools/check_rust.py`, `tools/tests/test_rust_gates.py`, `tools/vendor.json`, and reviewed translation fingerprints. Expected linked devguide paths are limited to the Rust dependency validator, its regression tests, and repository-set records needed to validate this exact first-party consumer. Each member's shared-tool copy and vendor hash update is confined to the linked Rust validator/test files. No other member product behavior changes.

## Acceptance-to-check map

- F1 create/inspect valid state and reject invalid input: consumer test against the pinned native service, asserting stable IDs and explicit error.
- F2 authorized deterministic execution and denied action: observe ordered progress/result and verify denial causes no unauthorized state change.
- F3 stop semantics: check requested versus effective outcome and that a denial or late stop never fabricates cancellation or erases completed work.
- F4 restart/reopen: stop and restart the service on a private test data directory; verify task/execution IDs, revisions, result and events; exercise declared in-flight recovery behavior.
- F5 replay/stale revision: identical replay retains the original identity; changed payload and stale revision fail explicitly.
- F6 interrupted/incompatible persistence: use disposable private fixtures and verify explicit recovery/failure without claiming physical power-loss behavior.
- F7 bounds/unavailable capability: exceed supported limits and request an unavailable capability; assert bounded, typed errors and truthful terminal state.
- Native target evidence: execute the consumer and all F1–F7 cases on this ARM64 macOS host against the pinned core commit. Compilation, static CI, and service-level runtime behavior are reported separately.
- Contributor gates: exact examples `AGENTS.md` commands, implementation Rust gates, real populated PR validation, cross-repository integration, `git diff --check`, exact-head CI and a separate exact-head reviewer.

## Roles, budget, risks

- Architect, implementer and verifier: Codex in this task (GPT-6 family; exact serving model is not exposed in the shell environment). Tools: Rust/Cargo, Python, Git and GitHub CLI. Independent reviewer: unassigned until a separate exact-head review; no self-review will be described as independent.
- Finite budget: 120 active minutes, at most 3 targeted repair attempts, no new spending. No token budget requested.
- Risks: the frozen G17 path list omits the source-profile and linked devguide changes necessary for a Rust consumer; the current examples docs profile rejects Cargo workspaces, and the current Rust validator rejects external Cargo packages. Do not weaken those checks broadly. The public loopback session is non-human; effective human-authorized cancellation cannot be inferred from a denied consumer request. Storage interruption tests use disposable fixtures and do not establish physical power-loss acceptance.
- Exclusions: release/publication, license selection, protection or permission changes, new third-party dependencies, model/GPU/training work, credentials/private records in Git, unrelated Goals, deployment and spending.

## Verification evidence

- Exact core checkout: `b06b7bac1aa121a4fee81e9fd0f79e4cf2eeeb3f` on ARM64 macOS 27.0.0, Rust 1.99.0.
- Passed: `cargo test --workspace --locked` (5 unit tests); `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Passed native F1–F7 acceptance runner: valid create/inspect plus local invalid-input rejection; deterministic value 7/one-step success; non-human cancellation denied with `FORBIDDEN` and state unchanged; completed task, execution and event page recovered after graceful restart; unfinished accepted multi-step execution recovered as `OutcomeUnknown` with an explicit event and no replay; identical admission replay retains execution ID, changed payload fails `IDEMPOTENCY_CONFLICT`, stale state fails `REVISION_CONFLICT`; incompatible disposable journal fails closed with `JOURNAL_INVALID`; unavailable capability and oversized request return explicit errors.
- Limits: F6 checks an incompatible journal fixture, not physical interruption or power loss. F3 confirms denial by the stock non-human session and makes no claim about effective human-authorized cancellation.
- Passed locally: contributor Python checks, exact ARM64 macOS Rust profile, populated PR body validation, translation fingerprint update and `git diff --check`. Pending: current cross-repository pinned snapshot integration, exact-head GitHub CI after the F4 update, and a separate exact-head reviewer.
