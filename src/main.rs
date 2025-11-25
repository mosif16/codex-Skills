//! codex-skills: Route tasks to the right skill playbook.

mod commands;
mod config;
mod loader;
mod matching;
mod skill;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use include_dir::{Dir, include_dir};

use commands::{cmd_instructions, cmd_list, cmd_pick, cmd_show};
use config::Config;
use loader::{load_skills_with_fallback, materialize_skills};

#[derive(Parser, Debug)]
#[command(name = "codex-skills", about = "Route tasks to the right skill playbook.")]
struct Cli {
    /// Directory containing skill folders (each with SKILL.md)
    #[arg(long, default_value = "skills", global = true)]
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

    /// Validate skill files for correctness
    Validate {
        /// Fail on warnings (stricter validation)
        #[arg(long)]
        strict: bool,
    },

    /// Show statistics about loaded skills
    Stats,

    /// Search within skill content
    Search {
        /// Text to search for in skill bodies
        query: String,
        /// Show context around matches
        #[arg(long, short, default_value_t = 2)]
        context: usize,
    },
}

/// Embedded skills directory, compiled into the binary.
static EMBEDDED_SKILLS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/skills");

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load();

    // Use config for skills_dir if not overridden on command line
    let skills_dir = if cli.skills_dir == PathBuf::from("skills") {
        config.skills_dir.clone().unwrap_or(cli.skills_dir)
    } else {
        cli.skills_dir
    };

    // Handle init command before loading skills
    if let Command::Init { force } = cli.command {
        materialize_skills(&skills_dir, force, &EMBEDDED_SKILLS_DIR)?;
        println!("Bundled skills written to {}", skills_dir.display());
        return Ok(());
    }

    // Load skills with fallback to embedded
    let skills = load_skills_with_fallback(&skills_dir, &EMBEDDED_SKILLS_DIR)?;

    if skills.is_empty() {
        println!(
            "No skills found in {}. Add SKILL.md files to get started.",
            skills_dir.display()
        );
        return Ok(());
    }

    match cli.command {
        Command::List {
            brief,
            verbose,
            json,
            clip,
        } => {
            // Use config clip length if default was used
            let effective_clip = if clip == 80 {
                config.get_clip_length()
            } else {
                clip
            };
            cmd_list(&skills, brief, verbose, json, effective_clip);
        }
        Command::Pick { query, top, show } => {
            // Use config top value if default was used
            let effective_top = if top == 3 {
                config.get_default_top()
            } else {
                top
            };
            cmd_pick(&skills, &query, effective_top, show);
        }
        Command::Show { name } => {
            cmd_show(&skills, &name);
        }
        Command::Instructions => {
            cmd_instructions(&skills, &skills_dir);
        }
        Command::Validate { strict } => {
            cmd_validate(&skills, strict);
        }
        Command::Stats => {
            cmd_stats(&skills);
        }
        Command::Search { query, context } => {
            cmd_search(&skills, &query, context);
        }
        Command::Init { .. } => unreachable!(),
    }

    Ok(())
}

/// Execute the `validate` command.
fn cmd_validate(skills: &[skill::Skill], strict: bool) {
    let mut errors = 0;
    let mut warnings = 0;

    for skill in skills {
        let mut skill_warnings = Vec::new();
        let mut skill_errors = Vec::new();

        // Check name
        if skill.name.is_empty() {
            skill_errors.push("Missing name".to_string());
        } else if skill.name.contains(' ') {
            skill_warnings.push("Name contains spaces (consider using kebab-case)".to_string());
        }

        // Check description
        if skill.summary.is_empty() {
            skill_errors.push("Missing description".to_string());
        } else if skill.summary.len() > 200 {
            skill_warnings.push(format!(
                "Description is {} chars (recommended: <200)",
                skill.summary.len()
            ));
        }

        // Check tags
        if skill.keywords.is_empty() {
            skill_warnings.push("No tags defined (recommended: 3+)".to_string());
        } else if skill.keywords.len() < 3 {
            skill_warnings.push(format!(
                "Only {} tag(s) (recommended: 3+)",
                skill.keywords.len()
            ));
        }

        // Check body content
        if skill.doc.is_empty() {
            skill_errors.push("Empty skill body".to_string());
        } else if skill.doc.len() < 100 {
            skill_warnings.push("Very short skill body (<100 chars)".to_string());
        }

        // Report issues
        if !skill_errors.is_empty() || !skill_warnings.is_empty() {
            println!("\n{}", skill.name);
            for err in &skill_errors {
                println!("  ✗ ERROR: {}", err);
                errors += 1;
            }
            for warn in &skill_warnings {
                println!("  ⚠ WARNING: {}", warn);
                warnings += 1;
            }
        }
    }

    println!("\n{} skills validated", skills.len());
    println!("  {} errors, {} warnings", errors, warnings);

    if errors > 0 || (strict && warnings > 0) {
        std::process::exit(1);
    }
}

/// Execute the `stats` command.
fn cmd_stats(skills: &[skill::Skill]) {
    println!("Skill Statistics");
    println!("{}", "-".repeat(40));
    println!("Total skills: {}", skills.len());

    // Find largest skill
    if let Some(largest) = skills.iter().max_by_key(|s| s.doc.len()) {
        println!(
            "Largest skill: {} ({} chars, {} extra docs)",
            largest.name,
            largest.doc.len(),
            largest.extra_docs.len()
        );
    }

    // Find smallest skill
    if let Some(smallest) = skills.iter().min_by_key(|s| s.doc.len()) {
        println!(
            "Smallest skill: {} ({} chars)",
            smallest.name,
            smallest.doc.len()
        );
    }

    // Count total extra docs
    let total_extra_docs: usize = skills.iter().map(|s| s.extra_docs.len()).sum();
    println!("Total extra docs: {}", total_extra_docs);

    // Average doc size
    let avg_size: usize = skills.iter().map(|s| s.doc.len()).sum::<usize>() / skills.len().max(1);
    println!("Average skill size: {} chars", avg_size);

    // Count skills with tags
    let with_tags = skills.iter().filter(|s| !s.keywords.is_empty()).count();
    println!("Skills with tags: {}/{}", with_tags, skills.len());

    // List all unique tags
    let mut all_tags: Vec<&str> = skills
        .iter()
        .flat_map(|s| s.keywords.iter().map(|k| k.as_str()))
        .collect();
    all_tags.sort();
    all_tags.dedup();
    println!("Unique tags: {}", all_tags.len());

    if !all_tags.is_empty() {
        println!("\nTags: {}", all_tags.join(", "));
    }
}

/// Execute the `search` command.
fn cmd_search(skills: &[skill::Skill], query: &str, context_lines: usize) {
    let query_lower = query.to_lowercase();
    let mut total_matches = 0;

    for skill in skills {
        let mut skill_matches = Vec::new();

        // Search in main doc
        for (line_num, line) in skill.doc.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                skill_matches.push((line_num, line.to_string(), "doc"));
            }
        }

        // Search in extra docs
        for extra in &skill.extra_docs {
            for (line_num, line) in extra.contents.lines().enumerate() {
                if line.to_lowercase().contains(&query_lower) {
                    skill_matches.push((line_num, line.to_string(), extra.name.as_str()));
                }
            }
        }

        if !skill_matches.is_empty() {
            println!("\n{} ({} matches)", skill.name, skill_matches.len());
            println!("{}", "-".repeat(40));

            for (line_num, line, source) in &skill_matches {
                let source_prefix = if *source == "doc" {
                    String::new()
                } else {
                    format!("[{}] ", source)
                };
                println!("  {}L{}: {}", source_prefix, line_num + 1, line.trim());

                // Show context if requested
                if context_lines > 0 {
                    let doc_content = if *source == "doc" {
                        &skill.doc
                    } else {
                        skill
                            .extra_docs
                            .iter()
                            .find(|e| e.name.as_str() == *source)
                            .map(|e| &e.contents)
                            .unwrap_or(&skill.doc)
                    };

                    let lines: Vec<&str> = doc_content.lines().collect();
                    let start = line_num.saturating_sub(context_lines);
                    let end = (*line_num + context_lines + 1).min(lines.len());

                    if start < *line_num || end > *line_num + 1 {
                        for i in start..end {
                            if i != *line_num {
                                println!("    L{}: {}", i + 1, lines[i].trim());
                            }
                        }
                    }
                }
            }
            total_matches += skill_matches.len();
        }
    }

    if total_matches == 0 {
        println!("No matches found for '{}'", query);
    } else {
        println!("\n{} total matches across {} skills", total_matches,
            skills.iter().filter(|s| {
                s.doc.to_lowercase().contains(&query_lower) ||
                s.extra_docs.iter().any(|e| e.contents.to_lowercase().contains(&query_lower))
            }).count()
        );
    }
}
