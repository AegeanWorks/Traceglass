use traceglass::ui::{parse_action, render};
use traceglass::{parse_trace, Comparison, Navigation, NavigationAction};

#[test]
fn commands_are_explicit_and_unknown_commands_do_not_move() {
    assert_eq!(parse_action("n"), Some(NavigationAction::Next));
    assert_eq!(
        parse_action("next-diff"),
        Some(NavigationAction::NextDifference)
    );
    assert_eq!(parse_action("q"), Some(NavigationAction::Quit));
    assert_eq!(parse_action("bogus"), None);
}

#[test]
fn rendering_labels_rows_and_escapes_terminal_control_characters() {
    let left = parse_trace("open\told\n").unwrap();
    let right = parse_trace("open\tnew\u{1b}[2J\n").unwrap();
    let comparison = Comparison::try_aligned(left, right, 16).unwrap();
    let output = render(&comparison, Navigation::new(&comparison));
    assert!(output.contains("changed"));
    assert!(output.contains("old"));
    assert!(output.contains("new\\u{1b}[2J"));
    assert!(!output.contains('\u{1b}'));
}

#[test]
fn long_values_are_visibly_clipped() {
    let trace = format!("event\t{}\n", "x".repeat(500));
    let comparison = Comparison::try_aligned(parse_trace(&trace).unwrap(), vec![], 10).unwrap();
    let output = render(&comparison, Navigation::new(&comparison));
    assert!(output.contains("…"));
    assert!(!output.contains(&"x".repeat(500)));
}

