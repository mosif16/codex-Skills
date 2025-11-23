# codex-skills

Small CLI that routes tasks to skill playbooks stored as `SKILL.md` files. The binary ships with two bundled skills (`brand-guidelines`, `frontend-design`) and can load any additional skills you add to a skills directory.

## Install
- From this repo: `cargo install --path . --force`
- Confirm: `codex-skills list`

## Quickstart (correct flag order)
The global options (like `--skills-dir`) must appear **before** the subcommand.
```bash
# use default ./skills folder
codex-skills init --force
codex-skills instructions
codex-skills list
codex-skills pick "your task description" --top 3 --show
codex-skills show "<skill-name>"

# use a custom skills directory
codex-skills --skills-dir /path/to/skills init --force
codex-skills --skills-dir /path/to/skills list
```
You can also set `SKILLS_DIR=/path/to/skills` instead of passing `--skills-dir`.

## Adding a new skill
1) Create a folder under `skills/` with a slugged name (e.g., `skills/my-new-skill`).  
2) Add a `SKILL.md` file with YAML frontmatter followed by the playbook body:
```markdown
---
name: My New Skill
description: One-sentence summary of what this skill covers.
tags:
  - keyword1
  - keyword2
---
Write the detailed playbook here. Include step-by-step guidance the agent should follow.
```
3) Keep the file name `SKILL.md` (case-insensitive variants `skill.md` also load).  
4) Test loading: `codex-skills list` and `codex-skills show "My New Skill"`.

Notes:
- The CLI searches recursively under the skills directory for `SKILL.md` files.
- `init --force` writes the bundled example skills; it won’t overwrite your additions unless they share the same paths.
- When embedding new default skills into the binary, place them in `skills/` and rebuild (`cargo install --path . --force`).

## Troubleshooting
- “unexpected argument '--skills-dir'”: move the flag before the subcommand (see Quickstart).
- “No skills found in skills”: ensure your `SKILL.md` files exist and are readable; run `codex-skills list` from the directory containing `skills/` or point `--skills-dir` to it.
