# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

codex-skills is a Rust CLI that routes tasks to skill playbooks stored as `SKILL.md` files. It uses token-based matching with Jaro-Winkler similarity scoring to find the best skill for a given query.

## Build & Development Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Release build

# Run tests
cargo test               # Run all tests
cargo test <test_name>   # Run a specific test

# Install locally
cargo install --path . --force

# Run the CLI directly
./target/release/codex-skills list
./target/debug/codex-skills pick "query" --top 3 --show
```

## Architecture

**Multi-module CLI** organized in `src/`:
- `main.rs`: Entry point with `Cli` / `Command` structs (Clap-based argument parsing)
- `skill.rs`: `Skill`, `ExtraDoc`, `SkillFrontmatter` data structures and parsing logic
- `matching.rs`: `SkillSignals` scoring system and `compute_signals()` with Jaro-Winkler similarity
- `loader.rs`: `load_skills()`, `load_embedded_skills()`, filesystem and embedded loading paths
- `commands.rs`: Command implementations (`cmd_list`, `cmd_pick`, `cmd_show`, etc.)
- `config.rs`: Configuration file support (`.codex-skills.toml`)

**Skill format**: Each skill is a markdown file with YAML frontmatter:
```yaml
---
name: skill-name
description: One-sentence summary
tags:
  - keyword1
  - keyword2
---
Playbook body content...
```

**Embedded skills**: Skills in `skills/` are compiled into the binary via `include_dir!()`. The `init` command materializes them to disk.

## Key Implementation Details

- **Scoring weights** (in `matching.rs` `SkillSignals::total_score()`): NAME=8, SUMMARY=5, TAG=4, BODY=1, PHRASE=1, NAME_SIM=2, SUMMARY_SIM=1
- **Similarity gating**: Jaro-Winkler scores only apply when there are token hits OR very high similarity (≥0.92 name, ≥0.94 summary)
- **Pre-computed tokens**: Skills cache tokenized name/summary/tags/body at load time for faster matching
- **Extra docs**: Other `.md` files in a skill folder are loaded as `extra_docs` and displayed with `--show`
- **Deduplication**: Skills are deduped by case-insensitive name
- **Config files**: Supports `.codex-skills.toml`, `codex-skills.toml`, or `~/.config/codex-skills/config.toml`

## Testing

Tests use `assert_cmd` with `cargo_bin_cmd!("codex-skills")` macro. Test files:
- `tests/pick_matching.rs`: Query-to-skill matching behavior
- `tests/pick_regressions.rs`: Edge cases and signal output
- `tests/list_output.rs`: List formatting options
- `tests/pick_show_full_content.rs`: Full content display

Pattern for test helpers:
```rust
fn pick(args: &[&str]) -> String {
    let mut cmd = cargo_bin_cmd!("codex-skills");
    cmd.arg("--skills-dir").arg("skills").args(["pick"]).args(args);
    String::from_utf8(cmd.assert().get_output().stdout.clone()).unwrap()
}
```

## CLI Commands

```bash
codex-skills list [--brief|--verbose|--json] [--clip N]
codex-skills pick "<query>" --top N [--show]
codex-skills show "<skill-name>"
codex-skills instructions
codex-skills init [--force]
codex-skills validate [--strict]
codex-skills stats
codex-skills search "<text>" [--context N]
```

Global option `--skills-dir` must appear **before** the subcommand.

## Configuration

Create `.codex-skills.toml` or `codex-skills.toml` in the project directory, or `~/.config/codex-skills/config.toml`:

```toml
default_top = 5       # Default number of results for pick
clip_length = 100     # Default summary clip length for list
skills_dir = "skills" # Default skills directory
```
