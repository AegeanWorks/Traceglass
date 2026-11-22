use traceglass::{
    compare, parse_trace, Comparison, Difference, Event, Navigation, NavigationAction,
};

#[test]
fn finds_first_changed_event_and_missing_tail() {
    let a = parse_trace("start\tready\nstep\tone\n").unwrap();
    let b = parse_trace("start\tready\nstep\ttwo\nend\tdone\n").unwrap();
    assert_eq!(compare(&a, &b), Some(Difference::Changed { index: 1 }));
    assert_eq!(
        compare(&a, &a[..1]),
        Some(Difference::MissingRight { index: 1 })
    );
}

#[test]
fn rejects_ambiguous_lines_and_accepts_empty_traces() {
    assert!(parse_trace("missing separator").is_err());
    assert!(parse_trace("\tvalue").is_err());
    assert_eq!(compare(&[], &[]), None);
}

#[test]
fn navigation_clamps_and_jumps_between_differences() {
    let left = parse_trace("a\t1\nb\t2\nc\t3\n").unwrap();
    let right = parse_trace("a\t1\nb\tx\nc\t3\nd\t4\n").unwrap();
    let comparison = Comparison::new(left, right);
    let state = Navigation::new(&comparison);
    assert_eq!(state.selected(), Some(0));
    let state = state.apply(NavigationAction::NextDifference, &comparison);
    assert_eq!(state.selected(), Some(1));
    let state = state.apply(NavigationAction::NextDifference, &comparison);
    assert_eq!(state.selected(), Some(3));
    let state = state.apply(NavigationAction::Next, &comparison);
    assert_eq!(state.selected(), Some(3));
    let state = state.apply(NavigationAction::PreviousDifference, &comparison);
    assert_eq!(state.selected(), Some(1));
    assert_eq!(
        comparison.row(3),
        Some((
            None,
            Some(&Event {
                name: "d".into(),
                value: "4".into()
            })
        ))
    );
}

#[test]
fn empty_navigation_is_safe_and_quit_is_terminal() {
    let comparison = Comparison::new(vec![], vec![]);
    let state = Navigation::new(&comparison).apply(NavigationAction::Next, &comparison);
    assert_eq!(state.selected(), None);
    let state = state.apply(NavigationAction::Quit, &comparison);
    assert!(state.is_finished());
    assert_eq!(state.apply(NavigationAction::Next, &comparison), state);
}

#[test]
fn navigation_reconciles_a_changed_snapshot_without_panicking() {
    let populated = Comparison::new(parse_trace("a\t1\nb\t2\n").unwrap(), vec![]);
    let empty = Comparison::new(vec![], vec![]);
    let state = Navigation::new(&populated).apply(NavigationAction::Next, &populated);
    assert_eq!(state.apply(NavigationAction::Next, &empty).selected(), None);
}
