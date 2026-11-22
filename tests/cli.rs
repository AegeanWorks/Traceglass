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
