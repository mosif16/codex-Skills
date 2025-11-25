//! Skill loading from filesystem and embedded resources.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use glob::{glob_with, MatchOptions};
use include_dir::Dir;

use crate::skill::{load_embedded_skills, load_extra_docs_fs, parse_skill, Skill};

/// Load skills from a filesystem directory.
/// Searches for SKILL.md files (case-insensitive) recursively.
pub fn load_skills(dir: &Path) -> Result<Vec<Skill>> {
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

    Ok(skills)
}

/// Load a single skill from a SKILL.md file path.
pub fn load_skill_md(path: &Path) -> Result<Option<Skill>> {
    let raw_text = fs::read_to_string(path)
        .with_context(|| format!("Failed to read skill file {}", path.display()))?;

    let extra_docs = if let Some(folder) = path.parent() {
        load_extra_docs_fs(folder, path)?
    } else {
        Vec::new()
    };

    parse_skill(&raw_text, path.display().to_string(), extra_docs)
}

/// Remove duplicate skills by name (case-insensitive).
pub fn dedupe_skills(skills: &mut Vec<Skill>) {
    let mut seen = HashSet::new();
    skills.retain(|s| seen.insert(s.name.to_lowercase()));
}

/// Load skills with fallback to embedded skills if none found on filesystem.
pub fn load_skills_with_fallback(
    skills_dir: &Path,
    embedded_dir: &Dir<'static>,
) -> Result<Vec<Skill>> {
    let fs_skills = if skills_dir.exists() {
        load_skills(skills_dir)?
    } else {
        Vec::new()
    };

    let mut skills = if fs_skills.is_empty() {
        load_embedded_skills(embedded_dir)?
    } else {
        fs_skills
    };

    dedupe_skills(&mut skills);
    Ok(skills)
}

/// Materialize embedded skills to the filesystem.
pub fn materialize_skills(dir: &Path, force: bool, embedded_dir: &Dir) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedupe_skills_removes_duplicates() {
        let mut skills = vec![
            create_test_skill("test-skill"),
            create_test_skill("Test-Skill"), // Same name, different case
            create_test_skill("other-skill"),
        ];
        dedupe_skills(&mut skills);
        assert_eq!(skills.len(), 2);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[1].name, "other-skill");
    }

    fn create_test_skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            summary: "Test summary".to_string(),
            keywords: vec![],
            doc: "Test doc".to_string(),
            extra_docs: vec![],
            name_tokens: vec![],
            summary_tokens: vec![],
            tag_tokens: vec![],
            body_tokens: vec![],
        }
    }
}
