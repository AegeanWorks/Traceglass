use std::process::Command;

#[test]
fn scripted_demo_reports_a_real_alignment_and_replays() {
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_traceglass"))
            .args(["demo", "--scripted", "--seed", "42"])
            .output()
            .unwrap()
    };
    let a = run();
    let b = run();
    assert!(a.status.success());
    assert_eq!(a.stdout, b.stdout);
    assert!(String::from_utf8(a.stdout)
        .unwrap()
        .contains("WORLD-DELTA: aligned_rows=2->3"));
}
