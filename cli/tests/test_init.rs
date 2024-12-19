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

#[test]
fn test_multiple_init() {
    let test_env = TestEnvironment::default();
    let (stdout, stderr) = test_env.jj_cmd_ok(test_env.env_root(), &["yak", "init", "repo1"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r#"Initialized repo in "repo1""#);
    let repo1_path = test_env.env_root().join("repo1");

    let (stdout, stderr) = test_env.jj_cmd_ok(test_env.env_root(), &["yak", "init", "repo2"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r#"Initialized repo in "repo2""#);
    let repo2_path = test_env.env_root().join("repo2");

    let stdout = test_env.jj_cmd_success(&repo1_path, &["log"]);
    insta::assert_snapshot!(stdout, @r"
    @  qpvuntsm test.user@example.com 2001-02-03 08:05:07 b4e46adb
    │  (empty) (no description set)
    ◆  zzzzzzzz root() 00000000
    ");

    let stdout = test_env.jj_cmd_success(&repo2_path, &["log"]);
    insta::assert_snapshot!(stdout, @r"
    @  rlvkpnrz test.user@example.com 2001-02-03 08:05:08 029ed36b
    │  (empty) (no description set)
    ◆  zzzzzzzz root() 00000000
    ");

    let stdout = test_env.jj_cmd_success(&repo2_path, &["yak", "status"]);
    insta::assert_snapshot!(stdout, @r"
    $TEST_ENV/repo1
    $TEST_ENV/repo2
    ");
}

#[test]
fn test_repos_are_independent() {
    let test_env = TestEnvironment::default();
    let (stdout, stderr) = test_env.jj_cmd_ok(test_env.env_root(), &["yak", "init", "repo1"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r#"Initialized repo in "repo1""#);
    let repo1_path = test_env.env_root().join("repo1");

    let (stdout, stderr) = test_env.jj_cmd_ok(test_env.env_root(), &["yak", "init", "repo2"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r#"Initialized repo in "repo2""#);
    let repo2_path = test_env.env_root().join("repo2");

    let stdout = test_env.jj_cmd_success(&repo1_path, &["log"]);
    insta::assert_snapshot!(stdout, @r"
    @  qpvuntsm test.user@example.com 2001-02-03 08:05:07 b4e46adb
    │  (empty) (no description set)
    ◆  zzzzzzzz root() 00000000
    ");

    let stdout = test_env.jj_cmd_success(&repo2_path, &["log"]);
    insta::assert_snapshot!(stdout, @r"
    @  rlvkpnrz test.user@example.com 2001-02-03 08:05:08 029ed36b
    │  (empty) (no description set)
    ◆  zzzzzzzz root() 00000000
    ");

    test_env.jj_cmd_ok(&repo1_path, &["new"]);
    let stdout = test_env.jj_cmd_success(&repo1_path, &["log"]);
    insta::assert_snapshot!(stdout, @r"
    @  mzvwutvl test.user@example.com 2001-02-03 08:05:11 bada728f
    │  (empty) (no description set)
    ○  qpvuntsm test.user@example.com 2001-02-03 08:05:07 b4e46adb
    │  (empty) (no description set)
    ◆  zzzzzzzz root() 00000000
    ");

    let stdout = test_env.jj_cmd_success(&repo2_path, &["yak", "status"]);
    insta::assert_snapshot!(stdout, @r"
    $TEST_ENV/repo1
    $TEST_ENV/repo2
    ");
}
