//! Skill data structures and parsing logic.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use glob::glob;
use include_dir::Dir;
use serde::Deserialize;

/// A skill playbook loaded from a SKILL.md file.
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub summary: String,
    pub keywords: Vec<String>,
    pub doc: String,
    pub extra_docs: Vec<ExtraDoc>,
    // Pre-computed tokens for faster matching
    pub name_tokens: Vec<String>,
    pub summary_tokens: Vec<String>,
    pub tag_tokens: Vec<String>,
    pub body_tokens: Vec<String>,
}

/// Additional documentation file associated with a skill.
#[derive(Debug, Clone)]
pub struct ExtraDoc {
    pub name: String,
    pub contents: String,
}

/// YAML frontmatter structure for skill files.
#[derive(Debug, Deserialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Normalize text into tokens for matching.
/// Filters stopwords and splits on non-alphanumeric characters.
pub fn normalized_tokens(text: &str) -> Vec<String> {
    let stopwords: HashSet<&'static str> = [
        "the", "a", "an", "to", "and", "or", "for", "into", "with", "when", "of", "use", "be",
        "is", "are", "on", "in", "at", "this", "that",
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

/// Parse a skill from raw markdown text with YAML frontmatter.
pub fn parse_skill(raw_text: &str, origin: String, extra_docs: Vec<ExtraDoc>) -> Result<Option<Skill>> {
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
        .with_context(|| {
            // Provide detailed error context for YAML parsing failures
            let line_count = fm_lines.len();
            format!(
                "Invalid YAML frontmatter in {} (lines 2-{}).\n\
                 Expected format:\n\
                 ---\n\
                 name: skill-name\n\
                 description: Short description\n\
                 tags:\n\
                 - tag1\n\
                 - tag2\n\
                 ---",
                origin,
                line_count + 1
            )
        })?;

    let doc = body_lines.join("\n").trim().to_string();

    // Pre-compute tokens for faster matching
    let name_tokens = normalized_tokens(&frontmatter.name);
    let summary_tokens = normalized_tokens(&frontmatter.description);
    let tag_tokens: Vec<String> = frontmatter
        .tags
        .iter()
        .flat_map(|k| normalized_tokens(k))
        .collect();
    let body_tokens = normalized_tokens(&doc);

    Ok(Some(Skill {
        name: frontmatter.name,
        summary: frontmatter.description,
        keywords: frontmatter.tags,
        doc,
        extra_docs,
        name_tokens,
        summary_tokens,
        tag_tokens,
        body_tokens,
    }))
}

/// Load extra documentation files from a skill folder (recursive).
pub fn load_extra_docs_fs(folder: &Path, skill_path: &Path) -> Result<Vec<ExtraDoc>> {
    let mut extra_docs = Vec::new();

    // Use recursive glob pattern to find all .md files in subdirectories too
    let pattern = folder.join("**/*.md");
    for entry in glob(pattern.to_str().unwrap()).with_context(|| {
        format!(
            "Failed to glob extra markdown files in {}",
            folder.display()
        )
    })? {
        let p = entry?;
        // Skip the main SKILL.md file
        if p == skill_path {
            continue;
        }
        // Skip any nested SKILL.md files (they belong to other skills)
        if p.file_name()
            .map(|n| n.to_string_lossy().to_lowercase() == "skill.md")
            .unwrap_or(false)
        {
            continue;
        }
        let contents = fs::read_to_string(&p)
            .with_context(|| format!("Failed to read extra skill file {}", p.display()))?;

        // Include relative path from skill folder for better context
        let relative_name = p
            .strip_prefix(folder)
            .map(|rel| rel.to_string_lossy().to_string())
            .unwrap_or_else(|_| {
                p.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "extra.md".into())
            });
        extra_docs.push(ExtraDoc {
            name: relative_name,
            contents,
        });
    }
    extra_docs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(extra_docs)
}

/// Load embedded skills from an include_dir directory.
pub fn load_embedded_skills(dir: &Dir) -> Result<Vec<Skill>> {
    fn walk(d: &Dir, skills: &mut Vec<Skill>) -> Result<()> {
        let mut skill_md = None;
        let mut extras: Vec<ExtraDoc> = Vec::new();

        for file in d.files() {
            let name = file
                .path()
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.eq_ignore_ascii_case("SKILL.md") {
                skill_md = Some(file);
            } else if name.to_lowercase().ends_with(".md") {
                if let Some(contents) = file.contents_utf8() {
                    extras.push(ExtraDoc {
                        name,
                        contents: contents.to_string(),
                    });
                }
            }
        }

        if let Some(skill_file) = skill_md {
            if let Some(contents) = skill_file.contents_utf8() {
                extras.sort_by(|a, b| a.name.cmp(&b.name));
                if let Some(skill) = parse_skill(
                    contents,
                    format!("embedded:{}", skill_file.path().display()),
                    extras,
                )? {
                    skills.push(skill);
                }
            }
        }

        for child in d.dirs() {
            walk(child, skills)?;
        }

        Ok(())
    }

    let mut skills = Vec::new();
    walk(dir, &mut skills)?;
    Ok(skills)
}

/// Find a skill by name (case-insensitive, supports partial match).
pub fn find_skill<'a>(skills: &'a [Skill], name: &str) -> Option<&'a Skill> {
    let needle = name.to_lowercase();
    skills
        .iter()
        .find(|s| s.name.to_lowercase() == needle || s.name.to_lowercase().contains(&needle))
}
