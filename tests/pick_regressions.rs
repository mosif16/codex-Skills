use assert_cmd::cargo::cargo_bin_cmd;

fn pick(args: &[&str]) -> String {
    let mut cmd = cargo_bin_cmd!("codex-skills");
    cmd.arg("--skills-dir")
        .arg("skills")
        .args(["pick"])
        .args(args);
    String::from_utf8(cmd.assert().get_output().stdout.clone()).unwrap()
}

#[test]
fn zero_match_suggests_closest_names_instead_of_dumping_all() {
    let out = pick(&["quantum knitting", "--top", "1", "--show"]);

    assert!(
        out.contains("No good skill match"),
        "missing no-match message: {out}"
    );
    assert!(
        out.contains("Closest skill names:"),
        "should surface a short shortlist instead of dumping all skills: {out}"
    );
}

#[test]
fn ios_ux_queries_surface_correct_skill() {
    let out = pick(&["ios ux improvements", "--top", "1"]);
    assert!(
        out.starts_with("1. ios-ux-design"),
        "expected ios-ux-design to rank first, got: {out}"
    );
}

#[test]
fn show_output_includes_similarity_signals() {
    let out = pick(&["ios ux improvements", "--top", "1", "--show"]);
    assert!(
        out.contains("phrase bonus="),
        "missing phrase bonus signal: {out}"
    );
    assert!(
        out.contains("name similarity="),
        "missing name similarity signal: {out}"
    );
}

#[test]
fn respects_top_limit_in_default_output() {
    let out = pick(&["design", "--top", "2"]);

    assert!(out.contains("1."), "first entry should be present: {out}");
    assert!(out.contains("2."), "second entry should be present: {out}");
    assert!(
        !out.contains("3."),
        "top=2 should not print a third entry; got: {out}"
    );
}
