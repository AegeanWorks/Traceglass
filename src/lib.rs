//! Pure parsing and comparison of ordered trace events.

mod alignment;
pub mod input;
pub mod ui;
pub use alignment::{align, AlignedRow, AlignmentError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Difference {
    Changed { index: usize },
    MissingLeft { index: usize },
    MissingRight { index: usize },
}

/// Parse one event per line: a nonempty name, a tab, then a value.
pub fn parse_trace(text: &str) -> Result<Vec<Event>, String> {
    text.lines()
        .enumerate()
        .map(|(i, line)| {
            let (name, value) = line
                .split_once('\t')
                .ok_or_else(|| format!("line {}: expected a tab", i + 1))?;
            if name.is_empty() {
                return Err(format!("line {}: event name is empty", i + 1));
            }
            Ok(Event {
                name: name.to_owned(),
                value: value.to_owned(),
            })
        })
        .collect()
}

/// Treat each UTF-8 line, including blank lines, as an event with a shared name.
pub fn parse_lines(text: &str) -> Vec<Event> {
    text.lines()
        .map(|line| Event {
            name: "line".to_owned(),
            value: line.to_owned(),
        })
        .collect()
}

pub fn compare(left: &[Event], right: &[Event]) -> Option<Difference> {
    for (index, (a, b)) in left.iter().zip(right).enumerate() {
        if a != b {
            return Some(Difference::Changed { index });
        }
    }
    match left.len().cmp(&right.len()) {
        std::cmp::Ordering::Less => Some(Difference::MissingLeft { index: left.len() }),
        std::cmp::Ordering::Greater => Some(Difference::MissingRight { index: right.len() }),
        std::cmp::Ordering::Equal => None,
    }
}

/// Two immutable snapshots, compared by position or alignment without inferred causality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    left: Vec<Event>,
    right: Vec<Event>,
    alignment: Option<Vec<AlignedRow>>,
}

impl Comparison {
    pub fn new(left: Vec<Event>, right: Vec<Event>) -> Self {
        Self {
            left,
            right,
            alignment: None,
        }
    }

    pub fn try_aligned(
        left: Vec<Event>,
        right: Vec<Event>,
        max_cells: usize,
    ) -> Result<Self, AlignmentError> {
        let alignment = align(&left, &right, max_cells)?;
        Ok(Self {
            left,
            right,
            alignment: Some(alignment),
        })
    }

    pub fn len(&self) -> usize {
        self.alignment
            .as_ref()
            .map_or_else(|| self.left.len().max(self.right.len()), Vec::len)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return zero-based source event indices for an aligned or positional row.
    pub fn source_indices(&self, index: usize) -> Option<(Option<usize>, Option<usize>)> {
        if let Some(rows) = &self.alignment {
            return rows.get(index).map(|row| match *row {
                AlignedRow::Equal { left, right } | AlignedRow::Changed { left, right } => {
                    (Some(left), Some(right))
                }
                AlignedRow::LeftOnly { left } => (Some(left), None),
                AlignedRow::RightOnly { right } => (None, Some(right)),
            });
        }
        (index < self.len()).then(|| {
            (
                (index < self.left.len()).then_some(index),
                (index < self.right.len()).then_some(index),
            )
        })
    }

    pub fn row(&self, index: usize) -> Option<(Option<&Event>, Option<&Event>)> {
        self.source_indices(index).map(|(left, right)| {
            (
                left.and_then(|i| self.left.get(i)),
                right.and_then(|i| self.right.get(i)),
            )
        })
    }

    pub fn differs(&self, index: usize) -> bool {
        self.row(index).is_some_and(|(left, right)| left != right)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    Previous,
    Next,
    PreviousDifference,
    NextDifference,
    Quit,
}

/// Finished is terminal; an empty comparison has no selected row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Navigation {
    Browsing { selected: Option<usize> },
    Finished,
}

impl Navigation {
    pub fn new(comparison: &Comparison) -> Self {
        Self::Browsing {
            selected: (!comparison.is_empty()).then_some(0),
        }
    }

    pub fn selected(self) -> Option<usize> {
        match self {
            Self::Browsing { selected } => selected,
            Self::Finished => None,
        }
    }

    pub fn is_finished(self) -> bool {
        self == Self::Finished
    }

    pub fn apply(self, action: NavigationAction, comparison: &Comparison) -> Self {
        let Self::Browsing { selected } = self else {
            return self;
        };
        if action == NavigationAction::Quit {
            return Self::Finished;
        }
        if comparison.is_empty() {
            return Self::Browsing { selected: None };
        }
        let index = selected.unwrap_or(0).min(comparison.len() - 1);
        let next = match action {
            NavigationAction::Previous => index.saturating_sub(1),
            NavigationAction::Next => (index + 1).min(comparison.len() - 1),
            NavigationAction::PreviousDifference => (0..index)
                .rev()
                .find(|&i| comparison.differs(i))
                .unwrap_or(index),
            NavigationAction::NextDifference => (index + 1..comparison.len())
                .find(|&i| comparison.differs(i))
                .unwrap_or(index),
            NavigationAction::Quit => unreachable!(),
        };
        Self::Browsing {
            selected: Some(next),
        }
    }
}
