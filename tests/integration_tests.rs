use assert_cmd::Command;
use std::process::Output;
use std::str::from_utf8;
use strip_ansi_escapes::strip;

fn run_lr_with_args(args: &[&str]) -> Output {
    let output = Command::cargo_bin("lr")
        .unwrap()
        .args(args)
        .output()
        .expect("Failed to execute command");

    let stdout = strip(&output.stdout);
    let stderr = strip(&output.stderr);

    Output {
        stdout,
        stderr,
        ..output
    }
}

#[test]
fn list_dir() {
    let args = ["tests/inputs"];

    let output = run_lr_with_args(&args);
    let stdout = from_utf8(&output.stdout).expect("Output not valid UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("dir1"));
    assert!(stdout.contains("dir2"));
    assert!(!stdout.contains(".hidden_file.txt"));
}

#[test]
fn list_hidden_files() {
    let args = ["tests/inputs", "-a"];

    let output = run_lr_with_args(&args);
    let stdout = from_utf8(&output.stdout).expect("Output not valid UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("dir1"));
    assert!(stdout.contains("dir2"));
    assert!(stdout.contains(".hidden_file.txt"));
}

#[test]
fn list_files_in_long_form() {
    let args = ["tests/inputs", "-l"];

    let output = run_lr_with_args(&args);
    let stdout = from_utf8(&output.stdout).expect("Output not valid UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("drwxrwxr-x 2 renato renato 4096"));
    assert!(stdout.contains("dir1"));
    assert!(stdout.contains("dr-xr-xr-x 2 renato renato 4096"));
    assert!(stdout.contains("dir2"));
    assert!(stdout.contains("-rw-rw-r-- 1 renato renato 0"));
    assert!(stdout.contains("file1.txt"));
}

#[test]
fn list_hidden_files_in_long_form() {
    let args = ["tests/inputs", "-la"];

    let output = run_lr_with_args(&args);
    let stdout = from_utf8(&output.stdout).expect("Output not valid UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("drwxrwxr-x 2 renato renato 4096"));
    assert!(stdout.contains("dir1"));
    assert!(stdout.contains("dr-xr-xr-x 2 renato renato 4096"));
    assert!(stdout.contains("dir2"));
    assert!(stdout.contains("-rw-rw-r-- 1 renato renato 0"));
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("-r--r--r-- 1 renato renato 0"));
    assert!(stdout.contains(".hidden_file.txt"));
}
