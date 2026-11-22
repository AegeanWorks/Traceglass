use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn view_loads_two_files_and_accepts_navigation() {
    let dir = std::env::temp_dir().join(format!("traceglass-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(dir.clone());
    let a = dir.join("a.trace");
    let b = dir.join("b.trace");
    fs::write(&a, "start\tready\nend\tok\n").unwrap();
    fs::write(&b, "start\tready\nlog\textra\nend\tok\n").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_traceglass"))
        .arg("view")
        .arg(&a)
        .arg(&b)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"N\nq\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("[inserted]"));
    assert!(text.contains(">     2 [inserted]"));
}

#[test]
fn check_is_noninteractive_and_distinguishes_equal_different_and_error() {
    let run = |left: &str, right: &str| {
        Command::new(env!("CARGO_BIN_EXE_traceglass"))
            .args(["check", left, right])
            .stdin(Stdio::null())
            .output()
            .unwrap()
    };
    let equal = run("examples/before.trace", "examples/before.trace");
    assert_eq!(equal.status.code(), Some(0));
    assert_eq!(String::from_utf8(equal.stdout).unwrap(), "equal: 3 rows\n");
    let different = run("examples/before.trace", "examples/after.trace");
    assert_eq!(different.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(different.stdout).unwrap(),
        "different: 1 of 4 rows; first difference: row 2\n"
    );
    let error = run("examples/does-not-exist.trace", "examples/after.trace");
    assert_eq!(error.status.code(), Some(2));
    assert!(error.stdout.is_empty());
    assert!(!error.stderr.is_empty());
    let malformed = run("Cargo.toml", "examples/after.trace");
    assert_eq!(malformed.status.code(), Some(2));
    assert!(malformed.stdout.is_empty());
    assert!(String::from_utf8(malformed.stderr)
        .unwrap()
        .contains("invalid trace"));
}

#[test]
fn json_check_reports_versioned_results_and_preserves_exit_codes() {
    let run = |left: &str, right: &str| {
        Command::new(env!("CARGO_BIN_EXE_traceglass"))
            .args(["check", "--json", left, right])
            .stdin(Stdio::null())
            .output()
            .unwrap()
    };
    let equal = run("examples/before.trace", "examples/before.trace");
    assert_eq!(equal.status.code(), Some(0));
    assert_eq!(
        equal.stdout,
        b"{\"version\":1,\"equal\":true,\"rows\":3,\"differences\":0,\"first\":null}\n"
    );
    let different = run("examples/before.trace", "examples/after.trace");
    assert_eq!(different.status.code(), Some(1));
    assert_eq!(different.stdout, b"{\"version\":1,\"equal\":false,\"rows\":4,\"differences\":1,\"first\":{\"row\":2,\"left_line\":null,\"right_line\":2}}\n");
    let error = run("examples/missing.trace", "examples/after.trace");
    assert_eq!(error.status.code(), Some(2));
    assert!(error.stdout.is_empty());
    assert!(!error.stderr.is_empty());
}

#[test]
fn check_accepts_a_pipe_on_either_side_and_rejects_two_pipes() {
    let run = |args: &[&str], input: &[u8]| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_traceglass"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        child.wait_with_output().unwrap()
    };
    let before = fs::read("examples/before.trace").unwrap();
    let equal = run(&["check", "examples/before.trace", "-"], &before);
    assert_eq!(equal.status.code(), Some(0));
    assert_eq!(equal.stdout, b"equal: 3 rows\n");
    let different = run(&["check", "--json", "-", "examples/after.trace"], &before);
    assert_eq!(different.status.code(), Some(1));
    assert_eq!(different.stdout, b"{\"version\":1,\"equal\":false,\"rows\":4,\"differences\":1,\"first\":{\"row\":2,\"left_line\":null,\"right_line\":2}}\n");
    let invalid = run(&["check", "-", "examples/after.trace"], &[0xff]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("UTF-8"));
    let ambiguous = run(&["check", "-", "-"], b"");
    assert_eq!(ambiguous.status.code(), Some(2));
    assert!(ambiguous.stdout.is_empty());
    assert!(String::from_utf8_lossy(&ambiguous.stderr).contains("standard input"));
}

#[test]
fn raw_lines_work_with_files_pipes_and_json() {
    let dir = std::env::temp_dir().join(format!("traceglass-raw-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(dir.clone());
    let before = dir.join("before.log");
    let after = dir.join("after.log");
    fs::write(&before, "start\n\nfinish\n").unwrap();
    fs::write(&after, "start\nretry\n\nfinish\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_traceglass"))
        .args(["check", "--json", "--lines"])
        .arg(&before)
        .arg(&after)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(result.stdout, b"{\"version\":1,\"equal\":false,\"rows\":4,\"differences\":1,\"first\":{\"row\":2,\"left_line\":null,\"right_line\":2}}\n");
    let mut child = Command::new(env!("CARGO_BIN_EXE_traceglass"))
        .args(["check", "--lines"])
        .arg(&before)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"start\n\nfinish\n")
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(result.stdout, b"equal: 3 rows\n");
    let mut view = Command::new(env!("CARGO_BIN_EXE_traceglass"))
        .args(["view", "--lines"])
        .arg(&before)
        .arg(&after)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    view.stdin.take().unwrap().write_all(b"q\n").unwrap();
    let result = view.wait_with_output().unwrap();
    assert!(result.status.success());
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.contains("[inserted]"));
    assert!(output.contains("R line 2: line: retry"));
}

#[test]
fn long_shared_prefix_can_be_compared_with_json_locations() {
    let dir = std::env::temp_dir().join(format!("traceglass-prefix-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(dir.clone());
    let left = dir.join("left.trace");
    let right = dir.join("right.trace");
    let common = "event\tstable\n".repeat(1500);
    fs::write(&left, &common).unwrap();
    fs::write(&right, format!("{common}extra\tvalue\n")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_traceglass"))
        .args(["check", "--json"])
        .arg(&left)
        .arg(&right)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"{\"version\":1,\"equal\":false,\"rows\":1501,\"differences\":1,\"first\":{\"row\":1501,\"left_line\":null,\"right_line\":1501}}\n");
}

#[test]
fn check_json_all_reports_every_difference_with_flags_in_any_order() {
    let dir = std::env::temp_dir();
    let left = dir.join(format!("tg-all-left-{}.trace", std::process::id()));
    let right = dir.join(format!("tg-all-right-{}.trace", std::process::id()));
    fs::write(&left, "a\t1\nb\t1\nc\t1\n").unwrap();
    fs::write(&right, "a\t1\nb\t2\nc\t1\nd\t3\n").unwrap();
    let expected = b"{\"version\":2,\"equal\":false,\"rows\":4,\"differences\":2,\"items\":[{\"row\":2,\"left_line\":2,\"right_line\":2,\"kind\":\"changed\"},{\"row\":4,\"left_line\":null,\"right_line\":4,\"kind\":\"inserted\"}]}\n";
    for args in [
        vec!["check", "--json", "--all-differences"],
        vec!["check", "--all-differences", "--json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_traceglass"))
            .args(args)
            .arg(&left)
            .arg(&right)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, expected);
    }
    let _ = fs::remove_file(&left);
    let _ = fs::remove_file(&right);
}

#[test]
fn view_accepts_a_256_byte_command_and_rejects_257_bytes() {
    for (len, accepted) in [(256, true), (257, false)] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_traceglass"))
            .args(["view", "examples/before.trace", "examples/after.trace"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut input = vec![b'x'; len];
        input.extend_from_slice(b"\nq\n");
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.success(), accepted);
        let text = String::from_utf8_lossy(if accepted { &out.stdout } else { &out.stderr });
        assert!(text.contains(if accepted { "Unknown command" } else { "command exceeds 256 bytes" }));
    }
}

#[test]
fn all_differences_requires_json_and_help_exits_successfully() {
    let invalid = Command::new(env!("CARGO_BIN_EXE_traceglass"))
        .args([
            "check",
            "--all-differences",
            "examples/before.trace",
            "examples/after.trace",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("--json"));

    for flag in ["help", "--help"] {
        let output = Command::new(env!("CARGO_BIN_EXE_traceglass"))
            .arg(flag)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("Usage:") && text.contains("--all-differences"));
    }
}

#[cfg(any(unix, windows))]
#[test]
fn non_unicode_path_reports_an_error_instead_of_panicking() {
    #[cfg(unix)]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        std::ffi::OsString::from_vec(vec![b'x', 0xff])
    };
    #[cfg(windows)]
    let path = {
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide(&[b'x' as u16, 0xd800])
    };
    let result = Command::new(env!("CARGO_BIN_EXE_traceglass"))
        .arg("check")
        .arg(path)
        .arg("examples/after.trace")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
}

#[cfg(unix)]
#[test]
fn existing_non_unicode_path_can_be_compared() {
    use std::os::unix::ffi::OsStringExt;
    let path = std::env::temp_dir().join(std::ffi::OsString::from_vec(
        format!("traceglass-{}-", std::process::id())
            .bytes()
            .chain([0xff])
            .collect(),
    ));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let _cleanup = Cleanup(path.clone());
    fs::write(&path, "event\tvalue\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_traceglass"))
        .arg("check")
        .arg(&path)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(result.stdout, b"equal: 1 rows\n");
}
