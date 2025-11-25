//! Command implementations for the CLI.

use std::path::Path;

use crate::matching::{closest_skill_names, rank_skills};
use crate::skill::{find_skill, Skill};

/// Print a separator line.
pub fn separator() -> String {
    "-".repeat(40)
}

/// Clip a summary to a maximum length.
pub fn clip_summary(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }
    let clipped: String = text.chars().take(limit).collect();
    format!("{}...", clipped)
}

/// Execute the `list` command.
pub fn cmd_list(skills: &[Skill], brief: bool, verbose: bool, json: bool, clip: usize) {
    if json {
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        let json_output = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        println!("{}", json_output);
        return;
    }

    for skill in skills {
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

/// Execute the `pick` command.
pub fn cmd_pick(skills: &[Skill], query: &str, top: usize, show: bool) {
    let ranked = rank_skills(skills, query);

    if let Some((best_score, _, _)) = ranked.first() {
        if *best_score == 0 {
            let shortlist = closest_skill_names(skills, query, 5);
            println!(
                "No good skill match for '{}'. Try a broader or simpler description.\nClosest skill names: {}",
                query,
                if shortlist.is_empty() {
                    "(no close names found)".to_string()
                } else {
                    shortlist.join(", ")
                }
            );
            return;
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
            println!("\n{}\n{}\n", separator(), skill.doc.trim());
            println!(
                "Top match reasoning: name hits={}, summary hits={}, tag hits={}, body hits={}, phrase bonus={}, name similarity={}, summary similarity={}",
                signals.name_hits,
                signals.summary_hits,
                signals.tag_hits,
                signals.body_hits,
                signals.phrase_bonus,
                signals.name_similarity,
                signals.summary_similarity,
            );
            for extra in &skill.extra_docs {
                println!(
                    "\n{} {}\n{}\n",
                    separator(),
                    extra.name,
                    extra.contents.trim()
                );
            }
            shown = true;
        }
    }

    if show && !shown {
        println!("No matches to display; try a broader query.");
    }
}

/// Execute the `show` command.
pub fn cmd_show(skills: &[Skill], name: &str) {
    if let Some(skill) = find_skill(skills, name) {
        println!("{}\n{}\n", separator(), skill.doc.trim());
        for extra in &skill.extra_docs {
            println!(
                "\n{} {}\n{}\n",
                separator(),
                extra.name,
                extra.contents.trim()
            );
        }
    } else {
        println!(
            "Skill '{}' not found. Use `codex-skills list` to see available entries.",
            name
        );
    }
}

/// Execute the `instructions` command.
pub fn cmd_instructions(skills: &[Skill], skills_dir: &Path) {
    println!(
        "STRICT INSTRUCTIONS FOR AGENTS\n{}
Only use skill playbooks found in: {}",
        separator(),
        skills_dir.display()
    );
    println!(
        "1) The only allowed skills are listed below; do NOT invent new skills.\n\
         2) Always pick the best-matching skill before acting; if none fit, say so.\n\
         3) When using a skill, follow its playbook text verbatim; do not alter or remove steps.\n\
         4) Cite the skill name when responding (e.g., 'Using skill: <name>').\n\
         5) Do not read or write files outside the skills directory."
    );
    println!("{}\nALLOWED SKILLS:", separator());
    for skill in skills {
        println!("- {} — {}", skill.name, skill.summary);
    }
}
