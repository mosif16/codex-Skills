use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

fn pick(query: &str) -> String {
    let mut cmd = cargo_bin_cmd!("codex-skills");
    cmd.arg("--skills-dir")
        .arg("skills")
        .arg("pick")
        .arg(query)
        .arg("--top")
        .arg("1");
    let output = cmd.assert().get_output().stdout.clone();
    String::from_utf8(output).unwrap()
}

#[test]
fn picks_brainstorming_for_idea_refinement_queries() {
    let out = pick("refine rough idea into plan");
    assert!(out.contains("brainstorming"), "output was: {out}");
}

#[test]
fn falls_back_cleanly_when_no_match() {
    let mut cmd = cargo_bin_cmd!("codex-skills");
    cmd.arg("--skills-dir")
        .arg("skills")
        .arg("pick")
        .arg("quantum knitting")
        .arg("--top")
        .arg("1")
        .arg("--show");

    cmd.assert().success().stdout(
        predicates::str::contains("No good skill match")
            .or(predicates::str::contains("try a broader query")),
    );
}

#[test]
fn tags_and_summary_are_weighted_over_body() {
    let out = pick("frontend interface design");
    assert!(out.starts_with("1. frontend-design"), "got: {out}");
}

#[test]
fn pick_show_explains_why_top_skill_won() {
    let mut cmd = cargo_bin_cmd!("codex-skills");
    cmd.arg("--skills-dir")
        .arg("skills")
        .arg("pick")
        .arg("debug failing tests")
        .arg("--top")
        .arg("1")
        .arg("--show");

    cmd.assert()
        .stdout(predicates::str::contains("Top match reasoning"))
        .stdout(
            predicates::str::contains("name hits")
                .or(predicates::str::contains("tag hits"))
                .or(predicates::str::contains("summary hits")),
        );
}
