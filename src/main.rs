use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use glob::glob;
use include_dir::{Dir, include_dir};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(name = "skills", about = "Route tasks to the right skill playbook.")]
struct Cli {
    /// Directory containing skill folders (each with SKILL.md)
    #[arg(long, default_value = "skills")]
    skills_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// List all available skills with a short summary
    List,

    /// Suggest the best matching skills for a task description
    Pick {
        /// Free-form task description to match against skills
        query: String,
        /// Number of candidates to show
        #[arg(short, long, default_value_t = 3)]
        top: usize,
        /// Immediately print the full playbook for the top result
        #[arg(long)]
        show: bool,
    },

    /// Open a specific skill by name
    Show {
        /// Skill name (case-insensitive)
        name: String,
    },

    /// Print strict agent instructions and the allowed skill list
    Instructions,

    /// Write bundled example skills into the skills directory
    Init {
        /// Overwrite existing files
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Clone)]
struct Skill {
    name: String,
    summary: String,
    keywords: Vec<String>,
    doc: String,
}

#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    tags: Vec<String>,
}

impl Skill {
    fn score(&self, query: &str) -> usize {
        let q = query.to_lowercase();
        let mut score = 0usize;
        for word in q.split_whitespace() {
            let w = word.trim();
            if w.is_empty() {
                continue;
            }
            let w_lower = w.to_lowercase();
            if self.name.to_lowercase().contains(&w_lower) {
                score += 4;
            }
            if self.summary.to_lowercase().contains(&w_lower) {
                score += 3;
            }
            if self
                .keywords
                .iter()
                .any(|k| k.to_lowercase().contains(&w_lower))
            {
                score += 2;
            }
            if self.doc.to_lowercase().contains(&w_lower) {
                score += 1;
            }
        }
        score
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command;

    static EMBEDDED_SKILLS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/skills");

    // Materialize bundled skills so agents don't need to locate this repo.
    if let Command::Init { force } = command {
        materialize_skills(&cli.skills_dir, force, &EMBEDDED_SKILLS_DIR)?;
        println!("Bundled skills written to {}", cli.skills_dir.display());
        return Ok(());
    }

    materialize_skills(&cli.skills_dir, false, &EMBEDDED_SKILLS_DIR)?;
    let skills = load_skills(&cli.skills_dir)?;

    if skills.is_empty() {
        println!(
            "No skills found in {}. Add SKILL.md files to get started.",
            cli.skills_dir.display()
        );
        return Ok(());
    }

    match command {
        Command::List => {
            for skill in &skills {
                println!("- {} — {}", skill.name, skill.summary);
            }
        }
        Command::Pick { query, top, show } => {
            let mut ranked: Vec<(usize, &Skill)> =
                skills.iter().map(|s| (s.score(&query), s)).collect();
            ranked.sort_by(|a, b| b.0.cmp(&a.0));

            let mut shown = false;
            for (idx, (score, skill)) in ranked.iter().take(top).enumerate() {
                println!(
                    "{}. {} (score: {}) — {}",
                    idx + 1,
                    skill.name,
                    score,
                    skill.summary
                );
                if show && idx == 0 {
                    println!("\n{}\n{}\n", separator(), skill.doc.trim());
                    shown = true;
                }
            }

            if show && !shown {
                println!("No matches to display; try a broader query.");
            }
        }
        Command::Show { name } => {
            if let Some(skill) = find_skill(&skills, &name) {
                println!("{}\n{}\n", separator(), skill.doc.trim());
            } else {
                println!(
                    "Skill '{}' not found. Use `skills list` to see available entries.",
                    name
                );
            }
        }
        Command::Instructions => {
            print_instructions(&skills, &cli.skills_dir);
        }
        Command::Init { .. } => unreachable!(),
    }

    Ok(())
}

fn separator() -> String {
    "-".repeat(40)
}

fn load_skills(dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    // Anthropic skills: **/SKILL.md (case-insensitive-ish)
    for pattern in ["SKILL.md", "skill.md"] {
        let md_pattern = dir.join("**").join(pattern);
        for entry in glob(md_pattern.to_str().unwrap())
            .with_context(|| format!("Failed to read glob for {}", pattern))?
        {
            let path = entry?;
            if let Some(skill) = load_skill_md(&path)? {
                skills.push(skill);
            }
        }
    }

    // Deduplicate by name, keeping first occurrence
    let mut seen = Vec::<String>::new();
    skills.retain(|s| {
        let lower = s.name.to_lowercase();
        if seen.contains(&lower) {
            false
        } else {
            seen.push(lower);
            true
        }
    });

    Ok(skills)
}

fn load_skill_md(path: &Path) -> Result<Option<Skill>> {
    let raw_text = fs::read_to_string(path)
        .with_context(|| format!("Failed to read skill file {}", path.display()))?;

    // Expect frontmatter delimited by lines starting with ---
    let mut lines = raw_text.lines();
    let Some(first) = lines.next() else {
        return Ok(None);
    };
    if first.trim() != "---" {
        return Ok(None);
    }

    let mut fm_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_body = false;
    for line in lines {
        if !in_body && line.trim() == "---" {
            in_body = true;
            continue;
        }
        if in_body {
            body_lines.push(line);
        } else {
            fm_lines.push(line);
        }
    }

    if !in_body {
        return Ok(None);
    }

    let fm_str = fm_lines.join("\n");
    let frontmatter: SkillFrontmatter = serde_yaml::from_str(&fm_str)
        .with_context(|| format!("Invalid YAML frontmatter in {}", path.display()))?;

    let doc = body_lines.join("\n").trim().to_string();

    Ok(Some(Skill {
        name: frontmatter.name,
        summary: frontmatter.description,
        keywords: frontmatter.tags,
        doc,
    }))
}

fn find_skill<'a>(skills: &'a [Skill], name: &str) -> Option<&'a Skill> {
    let needle = name.to_lowercase();
    skills
        .iter()
        .find(|s| s.name.to_lowercase() == needle || s.name.to_lowercase().contains(&needle))
}

fn materialize_skills(dir: &Path, force: bool, embedded_dir: &Dir) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    // include_dir's files() is non-recursive; walk the tree manually so nested skill folders are written.
    fn write_dir(to: &Path, force: bool, d: &Dir) -> Result<()> {
        for file in d.files() {
            let target = to.join(file.path());
            if target.exists() && !force {
                continue;
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&target, file.contents())?;
        }
        for child in d.dirs() {
            write_dir(to, force, child)?;
        }
        Ok(())
    }

    write_dir(dir, force, embedded_dir)
}

fn print_instructions(skills: &[Skill], skills_dir: &Path) {
    println!(
        "STRICT INSTRUCTIONS FOR AGENTS\n{}
Only use skill playbooks found in: {}",
        separator(),
        skills_dir.display()
    );
    println!(
        "1) The only allowed skills are listed below; do NOT invent new skills.\n2) Always pick the best-matching skill before acting; if none fit, say so.\n3) When using a skill, follow its playbook text verbatim; do not alter or remove steps.\n4) Cite the skill name when responding (e.g., 'Using skill: <name>').\n5) Do not read or write files outside the skills directory."
    );
    println!("{}\nALLOWED SKILLS:", separator());
    for skill in skills {
        println!("- {} — {}", skill.name, skill.summary);
    }
}
