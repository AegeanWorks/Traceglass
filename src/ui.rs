//! Pure terminal presentation and command interpretation.
use crate::{Comparison, Event, Navigation, NavigationAction};

/// Produce a stable, non-interactive summary without performing I/O.
pub fn summarize(comparison: &Comparison) -> (bool, String) {
    let mut count = 0usize;
    let mut first = None;
    for index in 0..comparison.len() {
        if comparison.differs(index) {
            count += 1;
            first.get_or_insert(index + 1);
        }
    }
    match first {
        None => (false, format!("equal: {} rows\n", comparison.len())),
        Some(row) => (
            true,
            format!(
                "different: {count} of {} rows; first difference: row {row}\n",
                comparison.len()
            ),
        ),
    }
}

pub fn parse_action(command: &str) -> Option<NavigationAction> {
    match command.trim() {
        "p" | "previous" => Some(NavigationAction::Previous),
        "n" | "next" => Some(NavigationAction::Next),
        "P" | "prev-diff" => Some(NavigationAction::PreviousDifference),
        "N" | "next-diff" => Some(NavigationAction::NextDifference),
        "q" | "quit" => Some(NavigationAction::Quit),
        _ => None,
    }
}

fn safe(text: &str) -> String {
    let mut chars = text.chars();
    let mut output = String::new();
    for c in chars.by_ref().take(120) {
        if c.is_control()
            || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        {
            output.extend(c.escape_default());
        } else {
            output.push(c);
        }
    }
    if chars.next().is_some() {
        output.push('…');
    }
    output
}

fn event(event: Option<&Event>, source: Option<usize>) -> String {
    match (event, source) {
        (Some(event), Some(index)) => format!(
            "line {}: {}: {}",
            index + 1,
            safe(&event.name),
            safe(&event.value)
        ),
        _ => "(missing)".to_string(),
    }
}

fn value_difference(left: &str, right: &str) -> Option<String> {
    let mut a = left.chars();
    let mut b = right.chars();
    let mut position = 1usize;
    loop {
        let (x, y) = (a.next(), b.next());
        if x != y {
            let describe = |c: Option<char>| {
                c.map(|c| safe(&c.to_string()))
                    .unwrap_or_else(|| "(end)".into())
            };
            return Some(format!(
                "        first value difference at character {position}: L {} | R {}\n",
                describe(x),
                describe(y)
            ));
        }
        if x.is_none() {
            return None;
        }
        position += 1;
    }
}

pub fn render(comparison: &Comparison, navigation: Navigation) -> String {
    let mut out = format!("TRACEGLASS | {} aligned rows\n", comparison.len());
    if let Some(selected) = navigation.selected() {
        let start = selected.saturating_sub(3);
        let end = selected.saturating_add(4).min(comparison.len());
        for index in start..end {
            let (left, right) = comparison.row(index).expect("bounded row");
            let (left_index, right_index) = comparison.source_indices(index).expect("bounded row");
            let kind = if left.is_none() {
                "inserted"
            } else if right.is_none() {
                "removed"
            } else if left != right {
                "changed"
            } else {
                "equal"
            };
            let cursor = if index == selected { ">" } else { " " };
            out.push_str(&format!(
                "{cursor} {:>5} [{kind}]\n        L {}\n        R {}\n",
                index + 1,
                event(left, left_index),
                event(right, right_index)
            ));
            if index == selected {
                if let (Some(left), Some(right)) = (left, right) {
                    if let Some(detail) = value_difference(&left.value, &right.value) {
                        out.push_str(&detail);
                    }
                }
            }
        }
    } else if comparison.is_empty() {
        out.push_str("Both traces are empty.\n");
    }
    out.push_str("p/n: previous/next | P/N: previous/next difference | q: quit\n");
    out
}
