use crate::common::TestEnvironment;

#[test]
fn test_init_git_internal() {
    let test_env = TestEnvironment::default();
    let (stdout, stderr) = test_env.jj_cmd_ok(test_env.env_root(), &["cultivate", "init", "repo"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r#"Initialized repo in "repo""#);
}
