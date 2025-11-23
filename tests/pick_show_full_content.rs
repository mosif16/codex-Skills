use assert_cmd::Command;
use predicates::str;

#[test]
fn pick_show_includes_additional_markdown_files() {
    let mut cmd = Command::cargo_bin("codex-skills").unwrap();
    cmd.args([
        "--skills-dir",
        "skills",
        "pick",
        "systematic-debugging",
        "--top",
        "1",
        "--show",
    ]);
    cmd.assert()
        .success()
        .stdout(str::contains("Pressure Test 1: Emergency Production Fix"));
}
