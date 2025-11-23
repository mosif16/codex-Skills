use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn default_list_is_clipped() {
    let mut cmd = Command::cargo_bin("codex-skills").unwrap();
    cmd.arg("--skills-dir").arg("skills").arg("list");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("brainstorming"))
        .stdout(predicates::str::contains("incremental validation").not());
}

#[test]
fn verbose_list_shows_full_summary() {
    let mut cmd = Command::cargo_bin("codex-skills").unwrap();
    cmd.args(["--skills-dir", "skills", "list", "--verbose"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("incremental validation"));
}

#[test]
fn json_list_returns_names_array() {
    let mut cmd = Command::cargo_bin("codex-skills").unwrap();
    cmd.args(["--skills-dir", "skills", "list", "--json"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("["))
        .stdout(predicates::str::contains("\"brainstorming\""))
        .stdout(predicates::str::contains("incremental validation").not());
}
