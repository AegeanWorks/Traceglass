//! Pure terminal presentation and command interpretation.
use crate::{Comparison, Event, Navigation, NavigationAction};

fn difference_counts(comparison: &Comparison) -> (usize, Option<usize>) {
    let mut count = 0usize;
    let mut first = None;
    for index in 0..comparison.len() {
        if comparison.differs(index) {
            count += 1;
            first.get_or_insert(index);
        }
    }
    (count, first)
}

/// Produce a stable, non-interactive summary without performing I/O.
pub fn summarize(comparison: &Comparison) -> (bool, String) {
    let (count, first) = difference_counts(comparison);
    match first {
        None => (false, format!("equal: {} rows\n", comparison.len())),
        Some(row) => (
            true,
            format!(
                "different: {count} of {} rows; first difference: row {}\n",
                comparison.len(),
                row + 1
            ),
        ),
    }
}

/// Version 1 JSON uses one-based row and file line numbers, or null for a missing side.
pub fn summarize_json(comparison: &Comparison) -> (bool, String) {
    let (count, first) = difference_counts(comparison);
    let detail = match first {
        None => "null".to_string(),
        Some(index) => {
            let (left, right) = comparison.source_indices(index).expect("bounded row");
            let line = |value: Option<usize>| {
                value
                    .map(|i| (i + 1).to_string())
                    .unwrap_or_else(|| "null".into())
            };
            format!(
                "{{\"row\":{},\"left_line\":{},\"right_line\":{}}}",
                index + 1,
                line(left),
                line(right)
            )
        }
    };
    (
        count != 0,
        format!(
            "{{\"version\":1,\"equal\":{},\"rows\":{},\"differences\":{},\"first\":{}}}\n",
            count == 0,
            comparison.len(),
            count,
            detail
        ),
    )
}

/// Version 2 JSON reports every differing row with source locations and its kind.
pub fn summarize_json_all(comparison: &Comparison) -> (bool, String) {
    let line = |value: Option<usize>| {
        value.map(|i| (i + 1).to_string()).unwrap_or_else(|| "null".into())
    };
    let mut items = Vec::new();
    let mut count = 0usize;
    for index in 0..comparison.len() {
        if !comparison.differs(index) {
            continue;
        }
        let (left, right) = comparison.source_indices(index).expect("bounded row");
        let (kind, left_line, right_line) = match (left, right) {
            (None, Some(_)) => ("inserted", "null".to_string(), line(right)),
            (Some(_), None) => ("removed", line(left), "null".to_string()),
            (Some(_), Some(_)) => ("changed", line(left), line(right)),
            (None, None) => continue,
        };
        count += 1;
        items.push(format!(
            "{{\"row\":{},\"left_line\":{},\"right_line\":{},\"kind\":\"{kind}\"}}",
            index + 1,
            left_line,
            right_line
        ));
    }
    (
        count != 0,
        format!(
            "{{\"version\":2,\"equal\":{},\"rows\":{},\"differences\":{},\"items\":[{}]}}\n",
            count == 0,
            comparison.len(),
            count,
            items.join(",")
        ),
    )
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
