// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.

use std::process::Stdio;
use test_util as util;
use test_util::TempDir;
use util::assert_contains;

#[test]
fn init_subcommand_without_dir() {
  let temp_dir = TempDir::new();
  let cwd = temp_dir.path();
  let deno_dir = util::new_deno_dir();

  let mut deno_cmd = util::deno_cmd_with_deno_dir(&deno_dir);
  let output = deno_cmd
    .current_dir(cwd)
    .arg("init")
    .stdout(Stdio::piped())
    .spawn()
    .unwrap()
    .wait_with_output()
    .unwrap();
  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert_contains!(stdout, "Project initialized");
  assert!(!stdout.contains("cd"));
  assert_contains!(stdout, "deno run main.ts");
  assert_contains!(stdout, "deno test");

  let mut deno_cmd = util::deno_cmd_with_deno_dir(&deno_dir);
  let output = deno_cmd
    .current_dir(cwd)
    .env("NO_COLOR", "1")
    .arg("run")
    .arg("main.ts")
    .stdout(Stdio::piped())
    .spawn()
    .unwrap()
    .wait_with_output()
    .unwrap();
  assert!(output.status.success());
  assert_eq!(output.stdout, b"Add 2 + 3 = 5\n");

  let mut deno_cmd = util::deno_cmd_with_deno_dir(&deno_dir);
  let output = deno_cmd
    .current_dir(cwd)
    .env("NO_COLOR", "1")
    .arg("test")
    .stdout(Stdio::piped())
    .spawn()
    .unwrap()
    .wait_with_output()
    .unwrap();
  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert_contains!(stdout, "1 passed");
}

#[test]
fn init_subcommand_with_dir_arg() {
  let temp_dir = TempDir::new();
  let cwd = temp_dir.path();
  let deno_dir = util::new_deno_dir();

  let mut deno_cmd = util::deno_cmd_with_deno_dir(&deno_dir);
  let output = deno_cmd
    .current_dir(cwd)
    .arg("init")
    .arg("my_dir")
    .stdout(Stdio::piped())
    .spawn()
    .unwrap()
    .wait_with_output()
    .unwrap();
  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert_contains!(stdout, "Project initialized");
  assert_contains!(stdout, "cd my_dir");
  assert_contains!(stdout, "deno run main.ts");
  assert_contains!(stdout, "deno test");

  let mut deno_cmd = util::deno_cmd_with_deno_dir(&deno_dir);
  let output = deno_cmd
    .current_dir(cwd)
    .env("NO_COLOR", "1")
    .arg("run")
    .arg("my_dir/main.ts")
    .stdout(Stdio::piped())
    .spawn()
    .unwrap()
    .wait_with_output()
    .unwrap();
  assert!(output.status.success());
  assert_eq!(output.stdout, b"Add 2 + 3 = 5\n");

  let mut deno_cmd = util::deno_cmd_with_deno_dir(&deno_dir);
  let output = deno_cmd
    .current_dir(cwd)
    .env("NO_COLOR", "1")
    .arg("test")
    .arg("my_dir/main_test.ts")
    .stdout(Stdio::piped())
    .spawn()
    .unwrap()
    .wait_with_output()
    .unwrap();
  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert_contains!(stdout, "1 passed");
}
