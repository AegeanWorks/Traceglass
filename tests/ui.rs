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

#[test]
fn changed_values_explain_a_difference_beyond_the_clipped_prefix() {
    let prefix = "x".repeat(130);
    let left = parse_trace(&format!("event\t{prefix}A\n")).unwrap();
    let right = parse_trace(&format!("event\t{prefix}B\n")).unwrap();
    let comparison = Comparison::try_aligned(left, right, 10).unwrap();
    let output = render(&comparison, Navigation::new(&comparison));
    assert!(output.contains("first value difference at character 131"));
    assert!(output.contains("L A | R B"));
}

#[test]
fn view_shows_original_line_numbers_after_an_insertion() {
    let left = parse_trace("open\t1\nclose\t3\n").unwrap();
    let right = parse_trace("open\t1\nlog\textra\nclose\t3\n").unwrap();
    let comparison = Comparison::try_aligned(left, right, 100).unwrap();
    let output = render(&comparison, Navigation::new(&comparison));
    assert!(output.contains("L line 2: close: 3"));
    assert!(output.contains("R line 3: close: 3"));
    assert!(output.contains("L (missing)"));
    assert!(output.contains("R line 2: log: extra"));
}
