# Repository Guidelines

## Project Structure & Module Organization
- `bins/` includes runnable binaries such as `simple-node`; each binary wires together modules via flattened clap configs.
- `core/`, `modules/`, `primitives/`, `storage/`, and `specs/` house library crates and shared definitions; look for `lib.rs` files to understand public APIs.
- Module-specific logic lives under `modules/<name>/src`, with each crate re-exporting configuration, service handles, and behaviours via `lib.rs`.
- Tests usually live next to the Rust code (`mod tests` or `tests/` directories inside crates); cross-crate test harnesses should be placed in the owning crate.

## Build, Test, and Development Commands
- `cargo fmt` — enforces Rust formatting; run before other commands or configure a pre-commit hook.
- `cargo build` — compiles all workspace crates; run immediately after formatting to catch build issues.
- `cargo clippy` — catches lints and potential bugs; treat warnings as blockers.
- `cargo test` — executes unit and integration tests across the workspace; add `--package <name>` to scope runs.
- `cargo run --package simple-node --` — starts the simple node binary with environment variables or CLI args (flattened configs) to exercise module interactions.
- **Workflow requirement** — after every local change run `cargo fmt`, `cargo build`, `cargo clippy`, and `cargo test` in that order; list those commands and outcomes in commit/PR notes and push commits with English messages.
- **Mandatory commit and push workflow** — after completing code modifications and verifying with `cargo fmt`, `cargo build`, and `cargo clippy`, you MUST commit the changes with an English commit message and push to the remote repository. This ensures all changes are properly tracked and synchronized.

## Specification and Code Modification Guidelines
- **Separation of concerns** — unless explicitly requested, when modifying specifications (files in `specs/`), do not modify code implementations. When modifying code implementations, do not modify specifications. This separation ensures that specification changes and code changes can be reviewed and tracked independently.
- **Spec writing rules** — when writing or modifying specifications (files in `specs/`), you MUST follow the rules defined in `specs/0000-specs-guide.md`, including file naming (`NNNN-filename.md`), content structure (title, overview, detailed specifications, references), content requirements (e.g. optional examples, struct/function presentation), maintenance principles, and language requirements (English only).

## Coding Style & Naming Conventions
- Rust code follows standard `rustfmt` defaults (4-space indentation, trailing commas, `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants).
- Keep module/struct comments concise; use doc comments (`///`) on public APIs and explain intent rather than implementation details.
- Prefer explicit errors (`anyhow::Result`, `map_err`) for logging, and explain panic/crash-worthy conditions in comments.
- Clap argument names mimic config paths (`p2p.key.type`, `p2p.listen-addr`); match CLI/env names when adding new `Args`.
- Avoid aliasing with `as` for potentially conflicting types; prefer fully qualified Rust paths instead of introducing temporary names for imported items.
- **Cargo.toml Configuration**: All package `edition` and `version` fields must reference the workspace values using `edition.workspace = true` and `version.workspace = true` instead of hardcoding values. This ensures consistency across all crates in the workspace.

## Testing Guidelines
- Tests are written with the standard Rust test harness; use `#[cfg(test)] mod tests` or `tests/` directories.
- Name tests descriptively (e.g., `handles_missing_peer`); group related tests in submodules when sharing fixtures.
- Integration or behavior tests that exercise RPC/P2P modules should run via the binary (e.g., `cargo run --package simple-node -- --help`) or harness crates.
- Always re-run `cargo test` after changing shared crates to catch regressions early.

## Commit & Pull Request Guidelines
- Keep commits focused and imperatively titled, e.g., `Add rendezvous server toggle`, `Fix P2P key defaults`.
- **Commit message format**: The commit message should be a single sentence that summarizes the changes in this commit. Use imperative mood and be concise.
- Reference related issues when possible and document decisions in the commit body (why, not just what).
- PRs should include: summary of changes, testing commands executed (cite `cargo test` or `cargo clippy`), and any manual verification steps (e.g., “started binary with default config”).
- Highlight configuration changes (network ports, key paths) explicitly so reviewers know to update deployments.

## Security & Configuration Tips
- P2P modules expect `p2p.key.type` and `p2p.key.path`; defaults use `KeyFile` and `p2p_key.toml` in the working directory.
- Keep secrets out of repo; mount or inject keys at runtime rather than committing them.

## Communication Preferences
- After making code modifications, do not add an explicit summary of the changes to your final response unless explicitly asked.
- Remove all `#[allow(dead_code)]` annotations from code; if `cargo fmt` / `cargo build` / `cargo clippy` / `cargo test` emit warnings thereafter, you can leave them without further fixes.
