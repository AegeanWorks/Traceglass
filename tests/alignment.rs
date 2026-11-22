use traceglass::{
    align, parse_trace, AlignedRow, AlignmentError, Comparison, Navigation, NavigationAction,
};

#[test]
fn insertion_does_not_make_all_later_events_look_changed() {
    let left = parse_trace("open\t1\nparse\t2\nclose\t3\n").unwrap();
    let right = parse_trace("open\t1\nlog\textra\nparse\t2\nclose\t3\n").unwrap();
    assert_eq!(
        align(&left, &right, 100).unwrap(),
        vec![
            AlignedRow::Equal { left: 0, right: 0 },
            AlignedRow::RightOnly { right: 1 },
            AlignedRow::Equal { left: 1, right: 2 },
            AlignedRow::Equal { left: 2, right: 3 },
        ]
    );
}

#[test]
fn alignment_is_bounded_before_allocating() {
    let events = parse_trace("a\t1\nb\t2\n").unwrap();
    assert_eq!(
        align(&events, &events, 8),
        Err(AlignmentError::TooManyCells)
    );
    assert_eq!(align(&[], &[], 1).unwrap(), vec![]);
}

#[test]
fn same_named_event_with_changed_value_is_one_changed_row() {
    let left = parse_trace("load\told\n").unwrap();
    let right = parse_trace("load\tnew\n").unwrap();
    assert_eq!(
        align(&left, &right, 4).unwrap(),
        vec![AlignedRow::Changed { left: 0, right: 0 }]
    );
}

#[test]
fn navigation_uses_aligned_rows_after_an_insertion() {
    let left = parse_trace("open\t1\nclose\t3\n").unwrap();
    let right = parse_trace("open\t1\nlog\t2\nclose\t3\n").unwrap();
    let comparison = Comparison::try_aligned(left, right, 100).unwrap();
    let state = Navigation::new(&comparison).apply(NavigationAction::NextDifference, &comparison);
    assert_eq!(state.selected(), Some(1));
    assert_eq!(comparison.row(1).unwrap().0, None);
    assert!(!comparison.differs(2));
}
