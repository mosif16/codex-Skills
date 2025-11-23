use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use glob::{glob, glob_with, MatchOptions};
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use serde_json;

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
    List {
        /// Output only names
        #[arg(long)]
        brief: bool,
        /// Output full summaries (no clipping)
        #[arg(long)]
        verbose: bool,
        /// Output JSON array of skill names
        #[arg(long)]
        json: bool,
        /// Maximum characters for clipped summaries
        #[arg(long, default_value_t = 80, value_name = "N")]
        clip: usize,
    },

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
    extra_docs: Vec<ExtraDoc>,
}

#[derive(Debug, Clone)]
struct ExtraDoc {
    name: String,
    contents: String,
}

#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    tags: Vec<String>,
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
        Command::List { brief, verbose, json, clip } => {
            if json {
                let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
                let json = serde_json::to_string(&names)?;
                println!("{}", json);
                return Ok(());
            }

            for skill in &skills {
                if brief {
                    println!("- {}", skill.name);
                } else if verbose {
                    println!("- {} — {}", skill.name, skill.summary);
                } else {
                    let clipped = clip_summary(&skill.summary, clip);
                    println!("- {} — {}", skill.name, clipped);
                }
            }
        }
        Command::Pick { query, top, show } => {
            let q_tokens = normalized_tokens(&query);
            let mut ranked: Vec<(usize, &Skill, SkillSignals)> = skills
                .iter()
                .map(|s| {
                    let signals = compute_signals(s, &q_tokens);
                    (signals.total_score(), s, signals)
                })
                .collect();
            ranked.sort_by(|a, b| b.0.cmp(&a.0));

            if let Some((best_score, best_skill, _)) = ranked.first() {
                if *best_score == 0 {
                    println!(
                        "No good skill match for '{}'. Try a broader or simpler description.\nHint: skills available: {}",
                        query,
                        skills.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", ")
                    );
                    return Ok(());
                }
                if show {
                    println!(
                        "Top match reasoning: name hits={}, summary hits={}, tag hits={}, body hits={}",
                        compute_signals(best_skill, &q_tokens).name_hits,
                        compute_signals(best_skill, &q_tokens).summary_hits,
                        compute_signals(best_skill, &q_tokens).tag_hits,
                        compute_signals(best_skill, &q_tokens).body_hits,
                    );
                }
            }

            let mut shown = false;
            for (idx, (score, skill, signals)) in ranked.iter().take(top).enumerate() {
                println!(
                    "{}. {} (score: {}) — {}",
                    idx + 1,
                    skill.name,
                    score,
                    skill.summary
                );
                if show && idx == 0 {
                    println!(
                        "\n{}\n{}\n",
                        separator(),
                        skill.doc.trim()
                    );
                    println!(
                        "Top match reasoning: name hits={}, summary hits={}, tag hits={}, body hits={}",
                        signals.name_hits, signals.summary_hits, signals.tag_hits, signals.body_hits
                    );
                    for extra in &skill.extra_docs {
                        println!("\n{} {}\n{}\n", separator(), extra.name, extra.contents.trim());
                    }
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
                for extra in &skill.extra_docs {
                    println!("\n{} {}\n{}\n", separator(), extra.name, extra.contents.trim());
                }
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

#[derive(Debug, Clone, Default)]
struct SkillSignals {
    name_hits: usize,
    summary_hits: usize,
    tag_hits: usize,
    body_hits: usize,
}

impl SkillSignals {
    fn total_score(&self) -> usize {
        const NAME_WEIGHT: usize = 8;
        const SUMMARY_WEIGHT: usize = 5;
        const TAG_WEIGHT: usize = 4;
        const BODY_WEIGHT: usize = 1;

        NAME_WEIGHT * self.name_hits
            + SUMMARY_WEIGHT * self.summary_hits
            + TAG_WEIGHT * self.tag_hits
            + BODY_WEIGHT * self.body_hits
    }
}

fn normalized_tokens(text: &str) -> Vec<String> {
    let stopwords: HashSet<&'static str> = [
        "the", "a", "an", "to", "and", "or", "for", "into", "with", "when", "of",
        "use", "be", "is", "are", "on", "in", "at", "this", "that",
    ]
    .into_iter()
    .collect();

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter_map(|w| {
            let word = w.trim();
            if word.is_empty() || stopwords.contains(word) {
                None
            } else {
                Some(word.to_string())
            }
        })
        .collect()
}

fn overlap(query_tokens: &[String], target_tokens: &[String]) -> usize {
    let target: HashSet<&str> = target_tokens.iter().map(String::as_str).collect();
    query_tokens
        .iter()
        .filter(|q| target.contains(q.as_str()))
        .count()
}

fn compute_signals(skill: &Skill, query_tokens: &[String]) -> SkillSignals {
    let name_tokens = normalized_tokens(&skill.name);
    let summary_tokens = normalized_tokens(&skill.summary);
    let tag_tokens: Vec<String> = skill
        .keywords
        .iter()
        .flat_map(|k| normalized_tokens(k))
        .collect();
    let body_tokens = normalized_tokens(&skill.doc);

    SkillSignals {
        name_hits: overlap(query_tokens, &name_tokens),
        summary_hits: overlap(query_tokens, &summary_tokens),
        tag_hits: overlap(query_tokens, &tag_tokens),
        body_hits: overlap(query_tokens, &body_tokens),
    }
}

fn separator() -> String {
    "-".repeat(40)
}

fn load_skills(dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    // Anthropic skills: **/SKILL.md (case-insensitive-ish)
    let md_pattern = dir.join("**").join("SKILL.md");
    let glob_options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };
    for entry in glob_with(md_pattern.to_str().unwrap(), glob_options)
        .with_context(|| "Failed to read glob for SKILL.md (case-insensitive)")?
    {
        let path = entry?;
        if let Some(skill) = load_skill_md(&path)? {
            skills.push(skill);
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

    let mut extra_docs = Vec::new();
    if let Some(folder) = path.parent() {
        let pattern = folder.join("*.md");
        for entry in glob(pattern.to_str().unwrap())
            .with_context(|| format!("Failed to glob extra markdown files in {}", folder.display()))?
        {
            let p = entry?;
            if p.file_name()
                .map(|n| n.to_string_lossy().to_lowercase() == "skill.md")
                .unwrap_or(false)
            {
                continue;
            }
            let contents = fs::read_to_string(&p)
                .with_context(|| format!("Failed to read extra skill file {}", p.display()))?;
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "extra.md".into());
            extra_docs.push(ExtraDoc { name, contents });
        }
        extra_docs.sort_by(|a, b| a.name.cmp(&b.name));
    }

    Ok(Some(Skill {
        name: frontmatter.name,
        summary: frontmatter.description,
        keywords: frontmatter.tags,
        doc,
        extra_docs,
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

fn clip_summary(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }
    let clipped: String = text.chars().take(limit).collect();
    format!("{}...", clipped)
}
