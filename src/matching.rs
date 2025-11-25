//! Skill matching and scoring logic.

use std::collections::HashSet;

use strsim::jaro_winkler;

use crate::skill::{normalized_tokens, Skill};

/// Scoring signals used to rank skill matches.
#[derive(Debug, Clone, Default)]
pub struct SkillSignals {
    pub name_hits: usize,
    pub summary_hits: usize,
    pub tag_hits: usize,
    pub body_hits: usize,
    pub phrase_bonus: usize,
    pub name_similarity: usize,
    pub summary_similarity: usize,
}

impl SkillSignals {
    /// Calculate the total weighted score for this skill match.
    pub fn total_score(&self) -> usize {
        const NAME_WEIGHT: usize = 8;
        const SUMMARY_WEIGHT: usize = 5;
        const TAG_WEIGHT: usize = 4;
        const BODY_WEIGHT: usize = 1;
        const PHRASE_WEIGHT: usize = 1;
        const NAME_SIM_WEIGHT: usize = 2;
        const SUMMARY_SIM_WEIGHT: usize = 1;

        NAME_WEIGHT * self.name_hits
            + SUMMARY_WEIGHT * self.summary_hits
            + TAG_WEIGHT * self.tag_hits
            + BODY_WEIGHT * self.body_hits
            + PHRASE_WEIGHT * self.phrase_bonus
            + NAME_SIM_WEIGHT * self.name_similarity
            + SUMMARY_SIM_WEIGHT * self.summary_similarity
    }
}

/// Count how many query tokens appear in the target tokens.
pub fn overlap(query_tokens: &[String], target_tokens: &[String]) -> usize {
    let target: HashSet<&str> = target_tokens.iter().map(String::as_str).collect();
    query_tokens
        .iter()
        .filter(|q| target.contains(q.as_str()))
        .count()
}

/// Compute matching signals between a query and a skill.
pub fn compute_signals(skill: &Skill, query_tokens: &[String], query_phrase: &str) -> SkillSignals {
    // Use pre-computed cached tokens from the Skill struct
    let base_hits = overlap(query_tokens, &skill.name_tokens)
        + overlap(query_tokens, &skill.summary_tokens)
        + overlap(query_tokens, &skill.tag_tokens)
        + overlap(query_tokens, &skill.body_tokens);

    let name_sim_raw = jaro_winkler(&skill.name.to_lowercase(), query_phrase);
    let summary_sim_raw = jaro_winkler(&skill.summary.to_lowercase(), query_phrase);

    // Only trust similarity when we also have token agreement or the match is very strong.
    let similarity_gate = base_hits > 0 || name_sim_raw >= 0.92 || summary_sim_raw >= 0.94;
    let name_similarity = if similarity_gate {
        (name_sim_raw * 10.0).round() as usize
    } else {
        0
    };
    let summary_similarity = if similarity_gate {
        (summary_sim_raw * 8.0).round() as usize
    } else {
        0
    };

    let phrase_bonus = if skill.name.to_lowercase().contains(query_phrase)
        || skill.summary.to_lowercase().contains(query_phrase)
    {
        10
    } else {
        0
    };

    SkillSignals {
        name_hits: overlap(query_tokens, &skill.name_tokens),
        summary_hits: overlap(query_tokens, &skill.summary_tokens),
        tag_hits: overlap(query_tokens, &skill.tag_tokens),
        body_hits: overlap(query_tokens, &skill.body_tokens),
        phrase_bonus,
        name_similarity,
        summary_similarity,
    }
}

/// Rank skills by how well they match a query.
/// Returns a sorted vector of (score, skill reference, signals).
pub fn rank_skills<'a>(
    skills: &'a [Skill],
    query: &str,
) -> Vec<(usize, &'a Skill, SkillSignals)> {
    let q_tokens = normalized_tokens(query);
    let query_phrase = query.to_lowercase();

    let mut ranked: Vec<(usize, &Skill, SkillSignals)> = skills
        .iter()
        .map(|s| {
            let signals = compute_signals(s, &q_tokens, &query_phrase);
            (signals.total_score(), s, signals)
        })
        .collect();

    ranked.sort_by(|a, b| b.0.cmp(&a.0));
    ranked
}

/// Find closest skill names using Jaro-Winkler similarity.
/// Used when no good match is found.
pub fn closest_skill_names<'a>(skills: &'a [Skill], query: &str, limit: usize) -> Vec<&'a str> {
    let query_phrase = query.to_lowercase();
    let mut closest: Vec<(f64, &str)> = skills
        .iter()
        .map(|s| {
            (
                jaro_winkler(&s.name.to_lowercase(), &query_phrase),
                s.name.as_str(),
            )
        })
        .filter(|(sim, _)| *sim > 0.0)
        .collect();

    closest.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    closest.into_iter().take(limit).map(|(_, n)| n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap_counts_matching_tokens() {
        let query = vec!["swift".to_string(), "ios".to_string(), "app".to_string()];
        let target = vec!["ios".to_string(), "swift".to_string(), "development".to_string()];
        assert_eq!(overlap(&query, &target), 2);
    }

    #[test]
    fn test_overlap_empty_returns_zero() {
        let query = vec!["rust".to_string()];
        let target = vec!["python".to_string()];
        assert_eq!(overlap(&query, &target), 0);
    }

    #[test]
    fn test_skill_signals_total_score() {
        let signals = SkillSignals {
            name_hits: 1,
            summary_hits: 1,
            tag_hits: 1,
            body_hits: 1,
            phrase_bonus: 10,
            name_similarity: 5,
            summary_similarity: 4,
        };
        // 8*1 + 5*1 + 4*1 + 1*1 + 1*10 + 2*5 + 1*4 = 8 + 5 + 4 + 1 + 10 + 10 + 4 = 42
        assert_eq!(signals.total_score(), 42);
    }
}
