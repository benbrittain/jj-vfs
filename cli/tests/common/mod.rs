// Copyright 2020 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
};

use itertools::Itertools as _;
use rand::{thread_rng, Rng};
use regex::{Captures, Regex};
use tempdir::TempDir;

pub struct TestEnvironment {
    _temp_dir: TempDir,
    env_root: PathBuf,
    home_dir: PathBuf,

    // CLI
    cli_config_path: PathBuf,
    env_vars: HashMap<String, String>,
    config_file_number: RefCell<i64>,
    command_number: RefCell<i64>,

    // Daemon
    daemon_child: std::process::Child,
    daemon_port: usize,
    daemon_dir: PathBuf,
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        //let mut other = TempDir::new("").unwrap();
        //std::mem::swap(&mut self._temp_dir, &mut other);
        //std::mem::forget(other);

        self.daemon_child
            .kill()
            .expect("Failed to kill daemon process")
    }
}

impl Default for TestEnvironment {
    fn default() -> Self {
        let tmp_dir = TempDir::new("jj-test").unwrap();
        let env_root = tmp_dir.path().canonicalize().unwrap();

        let home_dir = env_root.join("home");
        std::fs::create_dir(&home_dir).unwrap();
        let config_dir = env_root.join("config");
        std::fs::create_dir(&config_dir).unwrap();
        let daemon_dir = env_root.join("daemon");
        std::fs::create_dir(&daemon_dir).unwrap();
        let daemon_dir_str = daemon_dir.to_str().unwrap();

        // Initialize a isolated daemon for this env
        let daemon_config = config_dir.join("daemon.toml");
        let daemon_port: usize = thread_rng().gen_range(11000..21000);
        std::fs::write(
            &daemon_config,
            format!(
                "grpc_addr = \"[::1]:{daemon_port}\"\ncache = \"{daemon_dir_str}\"\n[nfs]\nmin_port = 1100\nmax_port = 1200\n"
            ),
        )
        .expect("Failed to write daemon config toml for testing setup");

        let daemon = assert_cmd::cargo::cargo_bin("daemon");
        let mut command = std::process::Command::new(daemon);
        command.args(["--config", daemon_config.to_str().unwrap()]);
        let daemon_child = command
            .spawn()
            .expect("Failed to start daemon for integration test");
        std::thread::sleep(std::time::Duration::from_millis(100));

        let env_vars = HashMap::new();
        let env = Self {
            _temp_dir: tmp_dir,

            env_root,
            home_dir,

            cli_config_path: config_dir,
            env_vars,
            config_file_number: RefCell::new(0),
            command_number: RefCell::new(0),

            daemon_child,
            daemon_port,
            daemon_dir,
        };
        env.add_config(format!(r#"grpc_port = {daemon_port}"#).as_str());
        // Use absolute timestamps in the operation log to make tests independent of the
        // current time.
        env.add_config(
            r#"
[template-aliases]
'format_time_range(time_range)' = 'time_range.start() ++ " - " ++ time_range.end()'
        "#,
        );

        env
    }
}

impl TestEnvironment {
    pub fn jj_cmd(&self, current_dir: &Path, args: &[&str]) -> assert_cmd::Command {
        let mut cmd = assert_cmd::Command::cargo_bin("cli").unwrap();
        cmd.current_dir(current_dir);
        cmd.args(args);

        cmd.env_clear();
        cmd.env("COLUMNS", "100");
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }
        cmd.env("RUST_BACKTRACE", "1");
        cmd.env("HOME", self.home_dir.to_str().unwrap());
        cmd.env("JJ_CONFIG", self.cli_config_path.to_str().unwrap());
        cmd.env("JJ_USER", "Test User");
        cmd.env("JJ_EMAIL", "test.user@example.com");
        cmd.env("JJ_OP_HOSTNAME", "host.example.com");
        cmd.env("JJ_OP_USERNAME", "test-username");
        cmd.env("JJ_TZ_OFFSET_MINS", "660");
        cmd.env("JJ_DAEMON_PORT", self.daemon_port.to_string());

        let mut command_number = self.command_number.borrow_mut();
        *command_number += 1;
        cmd.env("JJ_RANDOMNESS_SEED", command_number.to_string());
        let timestamp = chrono::DateTime::parse_from_rfc3339("2001-02-03T04:05:06+07:00").unwrap();
        let timestamp = timestamp + chrono::Duration::try_seconds(*command_number).unwrap();
        cmd.env("JJ_TIMESTAMP", timestamp.to_rfc3339());
        cmd.env("JJ_OP_TIMESTAMP", timestamp.to_rfc3339());

        cmd
    }

    pub fn write_stdin(&self, cmd: &mut assert_cmd::Command, stdin: &str) {
        cmd.env("JJ_INTERACTIVE", "1");
        cmd.write_stdin(stdin);
    }

    pub fn jj_cmd_stdin(
        &self,
        current_dir: &Path,
        args: &[&str],
        stdin: &str,
    ) -> assert_cmd::Command {
        let mut cmd = self.jj_cmd(current_dir, args);
        self.write_stdin(&mut cmd, stdin);

        cmd
    }

    fn get_ok(&self, mut cmd: assert_cmd::Command) -> (String, String) {
        let assert = cmd.assert().success();
        let stdout = self.normalize_output(&get_stdout_string(&assert));
        let stderr = self.normalize_output(&get_stderr_string(&assert));
        (stdout, stderr)
    }

    /// Run a `jj` command, check that it was successful, and return its
    /// `(stdout, stderr)`.
    pub fn jj_cmd_ok(&self, current_dir: &Path, args: &[&str]) -> (String, String) {
        self.get_ok(self.jj_cmd(current_dir, args))
    }

    pub fn jj_cmd_stdin_ok(
        &self,
        current_dir: &Path,
        args: &[&str],
        stdin: &str,
    ) -> (String, String) {
        self.get_ok(self.jj_cmd_stdin(current_dir, args, stdin))
    }

    /// Run a `jj` command, check that it was successful, and return its stdout
    #[track_caller]
    pub fn jj_cmd_success(&self, current_dir: &Path, args: &[&str]) -> String {
        let assert = self.jj_cmd(current_dir, args).assert().success().stderr("");
        self.normalize_output(&get_stdout_string(&assert))
    }

    /// Run a `jj` command, check that it failed with code 1, and return its
    /// stderr
    #[must_use]
    pub fn jj_cmd_failure(&self, current_dir: &Path, args: &[&str]) -> String {
        let assert = self.jj_cmd(current_dir, args).assert().code(1).stdout("");
        self.normalize_output(&get_stderr_string(&assert))
    }

    /// Run a `jj` command and check that it failed with code 2 (for invalid
    /// usage)
    #[must_use]
    pub fn jj_cmd_cli_error(&self, current_dir: &Path, args: &[&str]) -> String {
        let assert = self.jj_cmd(current_dir, args).assert().code(2).stdout("");
        self.normalize_output(&get_stderr_string(&assert))
    }

    /// Run a `jj` command, check that it failed with code 255, and return its
    /// stderr
    #[must_use]
    pub fn jj_cmd_internal_error(&self, current_dir: &Path, args: &[&str]) -> String {
        let assert = self.jj_cmd(current_dir, args).assert().code(255).stdout("");
        self.normalize_output(&get_stderr_string(&assert))
    }

    /// Run a `jj` command, check that it failed with code 101, and return its
    /// stderr
    #[must_use]
    #[allow(dead_code)]
    pub fn jj_cmd_panic(&self, current_dir: &Path, args: &[&str]) -> String {
        let assert = self.jj_cmd(current_dir, args).assert().code(101).stdout("");
        self.normalize_output(&get_stderr_string(&assert))
    }

    pub fn env_root(&self) -> &Path {
        &self.env_root
    }

    pub fn home_dir(&self) -> &Path {
        &self.home_dir
    }

    pub fn cli_config_path(&self) -> &PathBuf {
        &self.cli_config_path
    }

    pub fn set_cli_config_path(&mut self, cli_config_path: PathBuf) {
        self.cli_config_path = cli_config_path;
    }

    pub fn add_config(&self, content: &str) {
        if self.cli_config_path.is_file() {
            panic!("add_config not supported when cli_config_path is a file");
        }
        // Concatenating two valid TOML files does not (generally) result in a valid
        // TOML file, so we create a new file every time instead.
        let mut config_file_number = self.config_file_number.borrow_mut();
        *config_file_number += 1;
        let config_file_number = *config_file_number;
        std::fs::write(
            self.cli_config_path
                .join(format!("config{config_file_number:04}.toml")),
            content,
        )
        .unwrap();
    }

    pub fn add_env_var(&mut self, key: &str, val: &str) {
        self.env_vars.insert(key.to_string(), val.to_string());
    }

    pub fn current_operation_id(&self, repo_path: &Path) -> String {
        let id_and_newline =
            self.jj_cmd_success(repo_path, &["debug", "operation", "--display=id"]);
        id_and_newline.trim_end().to_owned()
    }

    /// Sets up the fake editor to read an edit script from the returned path
    /// Also sets up the fake editor as a merge tool named "fake-editor"
    pub fn set_up_fake_editor(&mut self) -> PathBuf {
        let editor_path = assert_cmd::cargo::cargo_bin("fake-editor");
        assert!(editor_path.is_file());
        // Simplified TOML escaping, hoping that there are no '"' or control characters
        // in it
        let escaped_editor_path = editor_path.to_str().unwrap().replace('\\', r"\\");
        self.add_env_var("EDITOR", &escaped_editor_path);
        self.add_config(&format!(
            r###"
                    [ui]
                    merge-editor = "fake-editor"

                    [merge-tools]
                    fake-editor.program="{escaped_editor_path}"
                    fake-editor.merge-args = ["$output"]
                "###
        ));
        let edit_script = self.env_root().join("edit_script");
        std::fs::write(&edit_script, "").unwrap();
        self.add_env_var("EDIT_SCRIPT", edit_script.to_str().unwrap());
        edit_script
    }

    /// Sets up the fake diff-editor to read an edit script from the returned
    /// path
    pub fn set_up_fake_diff_editor(&mut self) -> PathBuf {
        let escaped_diff_editor_path = escaped_fake_diff_editor_path();
        self.add_config(&format!(
            r###"
            ui.diff-editor = "fake-diff-editor"
            merge-tools.fake-diff-editor.program = "{escaped_diff_editor_path}"
            "###
        ));
        let edit_script = self.env_root().join("diff_edit_script");
        std::fs::write(&edit_script, "").unwrap();
        self.add_env_var("DIFF_EDIT_SCRIPT", edit_script.to_str().unwrap());
        edit_script
    }

    pub fn normalize_output(&self, text: &str) -> String {
        let text = text.replace("jj.exe", "jj");
        let regex = Regex::new(&format!(
            r"{}(\S+)",
            regex::escape(&self.env_root.display().to_string())
        ))
        .unwrap();
        regex
            .replace_all(&text, |caps: &Captures| {
                format!("$TEST_ENV{}", caps[1].replace('\\', "/"))
            })
            .to_string()
    }

    /// Used before mutating operations to create more predictable commit ids
    /// and change ids in tests
    ///
    /// `test_env.advance_test_rng_seed_to_multiple_of(200_000)` can be inserted
    /// wherever convenient throughout your test. If desired, you can have
    /// "subheadings" with steps of (e.g.) 10_000, 500, 25.
    pub fn advance_test_rng_seed_to_multiple_of(&self, step: i64) {
        assert!(step > 0, "step must be >0, got {step}");
        let mut command_number = self.command_number.borrow_mut();
        *command_number = step * (*command_number / step) + step;
    }
}

#[track_caller]
pub fn get_stdout_string(assert: &assert_cmd::assert::Assert) -> String {
    String::from_utf8(assert.get_output().stdout.clone()).unwrap()
}

#[track_caller]
pub fn get_stderr_string(assert: &assert_cmd::assert::Assert) -> String {
    String::from_utf8(assert.get_output().stderr.clone()).unwrap()
}

pub fn escaped_fake_diff_editor_path() -> String {
    let diff_editor_path = assert_cmd::cargo::cargo_bin("fake-diff-editor");
    assert!(diff_editor_path.is_file());
    // Simplified TOML escaping, hoping that there are no '"' or control characters
    // in it
    diff_editor_path.to_str().unwrap().replace('\\', r"\\")
}

/// Returns a string with the last line removed.
///
/// Use this to remove the root error message containing platform-specific
/// content for example.
pub fn strip_last_line(s: &str) -> &str {
    s.trim_end_matches('\n')
        .rsplit_once('\n')
        .map_or(s, |(h, _)| &s[..h.len() + 1])
}
