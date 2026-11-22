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
