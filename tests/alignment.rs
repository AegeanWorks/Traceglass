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

#[test]
fn source_indices_follow_each_original_trace_across_insertions_and_removals() {
    let left = parse_trace("open\t1\nremoved\t2\nclose\t3\n").unwrap();
    let right = parse_trace("open\t1\nadded\t2\nclose\t3\n").unwrap();
    let comparison = Comparison::try_aligned(left, right, 100).unwrap();
    assert_eq!(comparison.source_indices(0), Some((Some(0), Some(0))));
    assert_eq!(comparison.source_indices(1), Some((Some(1), None)));
    assert_eq!(comparison.source_indices(2), Some((None, Some(1))));
    assert_eq!(comparison.source_indices(3), Some((Some(2), Some(2))));
    assert_eq!(comparison.source_indices(4), None);
    let positional = Comparison::new(parse_trace("a\t1\nb\t2\n").unwrap(), vec![]);
    assert_eq!(positional.source_indices(1), Some((Some(1), None)));
}

#[test]
fn long_identical_prefix_does_not_consume_matrix_budget() {
    let left = parse_trace(&"event\tstable\n".repeat(1500)).unwrap();
    let mut right = left.clone();
    right.extend(parse_trace("extra\tvalue\n").unwrap());
    let comparison = Comparison::try_aligned(left, right, 2).unwrap();
    assert_eq!(comparison.len(), 1501);
    assert_eq!(
        comparison.source_indices(1499),
        Some((Some(1499), Some(1499)))
    );
    assert_eq!(comparison.source_indices(1500), Some((None, Some(1500))));
}

#[test]
fn remaining_unmatched_matrix_is_still_bounded() {
    let left = parse_trace("same\t1\na\t1\nb\t1\n").unwrap();
    let right = parse_trace("same\t1\nc\t1\nd\t1\n").unwrap();
    assert_eq!(
        Comparison::try_aligned(left, right, 8),
        Err(AlignmentError::TooManyCells)
    );
}

#[test]
fn prefix_optimization_preserves_full_matrix_ties_for_small_traces() {
    let symbols = parse_trace("a\t1\na\t2\nb\t1\n").unwrap();
    let mut traces = vec![Vec::new()];
    for _ in 0..3 {
        let previous = traces.clone();
        for trace in previous {
            for event in &symbols {
                let mut next = trace.clone();
                next.push(event.clone());
                traces.push(next);
            }
        }
    }
    for left in &traces {
        for right in &traces {
            let expected = align(left, right, 100).unwrap();
            let comparison = Comparison::try_aligned(left.clone(), right.clone(), 100).unwrap();
            let actual: Vec<_> = (0..comparison.len())
                .map(|i| comparison.source_indices(i).unwrap())
                .collect();
            let expected: Vec<_> = expected
                .iter()
                .map(|row| match row {
                    AlignedRow::Equal { left, right } | AlignedRow::Changed { left, right } => {
                        (Some(*left), Some(*right))
                    }
                    AlignedRow::LeftOnly { left } => (Some(*left), None),
                    AlignedRow::RightOnly { right } => (None, Some(*right)),
                })
                .collect();
            assert_eq!(actual, expected, "left={left:?}, right={right:?}");
        }
    }
}
