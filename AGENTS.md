# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs`: CLI entry for `codex-skills`; handles commands, skill loading, and matching logic.
- `skills/`: bundled and custom skills (`SKILL.md` per skill, optional extras via `extra_docs`).
- `tests/`: integration tests for CLI behaviors (list, pick, regressions).
- `Cargo.toml` / `Cargo.lock`: Rust crate metadata and locked dependencies.
- `target/`: build artifacts; not checked in.
- Keep new modules small and purpose-driven; prefer adding helpers to focused submodules over large single files.

## Build, Test, and Development Commands
- `cargo build` — debug build for rapid iteration.
- `cargo build --release` — optimized binary at `target/release/codex-skills`.
- `cargo fmt -- --check` — enforce formatting before commits.
- `cargo clippy -- -D warnings` — lint with warnings as errors.
- `cargo test` — run integration suite in `tests/`.
- Local run examples: `./target/debug/codex-skills list`, `./target/debug/codex-skills pick "triage bug" --top 3 --show`.

## Coding Style & Naming Conventions
- Rust 2021 style, 4-space indentation, line width ≤100.
- `snake_case` for functions/vars/modules; `UpperCamelCase` for types and skill names; `SCREAMING_SNAKE_CASE` for constants.
- Prefer early returns, clear error contexts, and avoid `unwrap` in production paths.
- All new public APIs need `///` docs with short examples when practical.

## Testing Guidelines
- Add integration tests under `tests/`; favor table-driven tests for CLI permutations.
- Name tests after behavior: `lists_brief_names`, `pick_shows_full_skill`.
- When adding new subcommands/flags, include failure cases (invalid args, missing skills).
- Run `cargo test` and include outcomes in PR notes.

## Commit & Pull Request Guidelines
- Commits: short imperative subject (`feat: add pick --json output`); scope per logical change.
- PRs: include summary, linked issue/ ticket, commands run (`cargo fmt`, `clippy`, `test`), and sample CLI output or screenshots when UX-facing.
- Keep diffs minimal; prefer separate PRs for refactors vs features.

## Security & Configuration Tips
- Avoid `codex-skills init --force` unless intentionally overwriting bundled skills.
- Validate skill content before merging (no secrets, licenses compatible).
- Use `SKILLS_DIR` or `--skills-dir` consistently in scripts; document defaults when sharing commands.
