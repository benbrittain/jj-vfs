use crate::common::TestEnvironment;

#[test]
fn test_init() {
    let test_env = TestEnvironment::default();
    let (stdout, stderr) = test_env.jj_cmd_ok(test_env.env_root(), &["yak", "init", "repo"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r#"Initialized repo in "repo""#);
    let repo_path = test_env.env_root().join("repo");

    let stdout = test_env.jj_cmd_success(&repo_path, &["log"]);
    insta::assert_snapshot!(stdout, @r"
    @  qpvuntsm test.user@example.com 2001-02-03 08:05:07 b4e46adb
    │  (empty) (no description set)
    ◆  zzzzzzzz root() 00000000
    ");
}
