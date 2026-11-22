//! Bounded deterministic alignment of ordered events.
use crate::Event;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlignedRow {
    Equal { left: usize, right: usize },
    Changed { left: usize, right: usize },
    LeftOnly { left: usize },
    RightOnly { right: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlignmentError {
    TooManyCells,
}

/// Find a minimum-cost alignment. Equal names may pair with different values;
/// unmatched names are insertions/removals. Ties prefer pairing, then removal.
pub fn align(
    left: &[Event],
    right: &[Event],
    max_cells: usize,
) -> Result<Vec<AlignedRow>, AlignmentError> {
    let rows = left
        .len()
        .checked_add(1)
        .ok_or(AlignmentError::TooManyCells)?;
    let cols = right
        .len()
        .checked_add(1)
        .ok_or(AlignmentError::TooManyCells)?;
    let cells = rows
        .checked_mul(cols)
        .filter(|&n| n <= max_cells)
        .ok_or(AlignmentError::TooManyCells)?;
    let mut cost = vec![0usize; cells];
    for i in (0..rows).rev() {
        for j in (0..cols).rev() {
            let at = i * cols + j;
            cost[at] = if i == left.len() {
                right.len() - j
            } else if j == right.len() {
                left.len() - i
            } else {
                let remove = 1 + cost[(i + 1) * cols + j];
                let insert = 1 + cost[i * cols + j + 1];
                let pair = if left[i].name == right[j].name {
                    usize::from(left[i] != right[j]) + cost[(i + 1) * cols + j + 1]
                } else {
                    usize::MAX
                };
                remove.min(insert).min(pair)
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut result = Vec::new();
    while i < left.len() || j < right.len() {
        if i < left.len()
            && j < right.len()
            && left[i].name == right[j].name
            && cost[i * cols + j] == usize::from(left[i] != right[j]) + cost[(i + 1) * cols + j + 1]
        {
            result.push(if left[i] == right[j] {
                AlignedRow::Equal { left: i, right: j }
            } else {
                AlignedRow::Changed { left: i, right: j }
            });
            i += 1;
            j += 1;
        } else if i < left.len() && cost[i * cols + j] == 1 + cost[(i + 1) * cols + j] {
            result.push(AlignedRow::LeftOnly { left: i });
            i += 1;
        } else {
            result.push(AlignedRow::RightOnly { right: j });
            j += 1;
        }
    }
    Ok(result)
}
