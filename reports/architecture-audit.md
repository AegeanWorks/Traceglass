# Architecture audit

## 2026-11-22

- `parse_trace`, `compare`, `align`, `Comparison`, and `Navigation::apply` operate on values and perform no I/O. The positional comparison shifts after an insertion; bounded minimum-cost alignment can preserve later matches, and navigation can use aligned rows. Neither mode proves causality.
- `input::read_trace` is an edge adapter: it caps bytes before decoding, rejects invalid UTF-8 and returns parsing errors rather than writing output. The shell opens files and displays errors.
- Navigation is an explicit `Browsing` or `Finished` state. Empty inputs and replacing the comparison with a shorter snapshot are covered by tests; `Finished` ignores later actions.
- `src/main.rs` owns file and console I/O; it reads each file once, bounds inputs and alignment work, handles EOF, and maps commands into pure navigation actions. The terminal view escapes control and bidirectional formatting characters and clips long fields. The Rust compatibility demo uses the same alignment core.
- A scripted end-to-end test covers two files and navigation. On 2026-11-22 the agent also ran the interface with an inserted event, inspected the rendered rows, and confirmed `N` selected the insertion and `q` exited. A physical terminal, other operating systems, and the declared minimum Rust version have not yet been tested.
- After forcing Cargo to recompile restored files, a second terminal run displayed the first difference at character 131 in clipped values; the corresponding regression test passed. Alignment remains heuristic for repeated names and is bounded to one million cells.
- A fresh Cargo target directory was used for `cargo test --locked`; all 17 tests passed on the Windows GNU toolchain. This is independent of the previous target's timestamp-based cache.

## Non-interactive checks

`check` and `view` share the same bounded input loader and alignment. The pure `summarize` function counts aligned differences; the shell writes its result and maps equal, different and error outcomes to exit statuses 0, 1 and 2. An end-to-end test covers both comparisons and input errors with stdin closed. The comparison still uses the documented heuristic for repeated event names.

`check --json` uses the same pure difference counts and comparison source indices as the text summary and terminal view. Its versioned schema uses one-based locations and explicit `null` for an absent side; it contains no event text. The shell alone performs output and maps errors to exit status 2.

`check` may read one trace from standard input using a bare `-`; the shell rejects two stdin operands before reading and uses the same bounded input adapter for files and pipes. `view` still reserves standard input for navigation and treats `-` as a filename. Subprocess tests cover both pipe positions, JSON mode, invalid UTF-8 and the ambiguous two-pipe case.

Raw `--lines` mode is explicitly selected at the shell edge. The same bounded UTF-8 reader feeds a pure parser that maps each input line to an event with the shared name `line`; comparison, navigation and rendering remain unchanged. Tests cover blank lines, limits, piped input, JSON and terminal viewing. Equal-cost alignment choices for repeated lines remain heuristic.

## Native paths

The shell now reads arguments as operating-system strings and passes file arguments as native paths into the file adapter; event parsing and alignment remain unchanged. A Windows test reproduces the former panic with an unpaired surrogate and verifies an ordinary input error instead. A Unix-only test exercises a real non-UTF-8 filename, but was not run on the Windows host.

## Source locations

`Comparison::source_indices` maps aligned rows to zero-based positions in the immutable input snapshots, including absent sides. The pure terminal renderer displays those positions as one-based file lines; a parsed event corresponds to exactly one input line. Tests cover insertions, removals, positional comparisons, and out-of-range indices.

## Shared prefix alignment

`Comparison::try_aligned` matches equal leading events before applying the matrix-cell budget to the remaining slices. It then offsets the residual source indices. The public `align` function remains the full-matrix reference with its original limit. Small-trace exhaustive comparisons check deterministic tie behavior; long-prefix and over-budget residual tests cover the new boundary. The optimization does not help traces whose first difference is near the beginning.

## Portfolio comparison

The public repository inventory was retrieved through GitHub's API on 2026-11-22 (one page of 100 was sufficient; page two was empty). Sextant already verifies state machines and checks golden traces for the first differing line. Traceglass compares generic program event traces and offers interactive inspection of bounded insertion/deletion-aware alignment. This distinguishes its workflow from Sextant; it is not a claim of global novelty.

The profile's paginated starred-repository names were also inspected. Descriptions of additional projects not present in the public owner inventory did not identify another trace comparison workbench.

## Complete JSON differences and CLI contract (2026-11-22, second session)

`ui::summarize_json_all` is a pure function that walks the aligned rows once and emits schema
version 2: `version`, `equal`, `rows`, `differences` and an `items` array. Each item carries the
one-based aligned row, both one-based source lines (`null` for an absent side) and a `kind` of
`changed`, `inserted` or `removed`, derived from which sides are present. It shares the same
comparison and source-index mapping as the text and v1 JSON summaries, so the three renderings
cannot disagree. Version 1 output and all exit codes are byte-identical; v2 is opt-in through
`check --all-differences`, which is refused without `--json`.

Argument parsing is now one order-independent parser that keeps file operands as native
`PathBuf` values until the shell opens them (ADR-004). `view` accepts `--lines`; `check` accepts
`--json`, `--lines` and `--all-differences` in any order, and rejects two identical flags by
treating the second as an operand and therefore a usage error. `help`, `--help` and `-h` write
the command summary to stdout with exit zero; other misuse still writes it to stderr with exit
two. The interactive command reader now measures the command bytes without the newline, so a
256-byte command is accepted and 257 bytes is rejected, matching the documented limit.

**Agent review of the running UI** (project rule G-002, 2026-11-22): with the GNU toolchain
selected, the compiled binary was exercised directly. `--help` printed the usage summary and
exited 0; `check --json --all-differences examples/before.trace examples/after.trace` emitted
v2 with one `inserted` item at row 2 and exited 1; reversing the flags and adding `--lines`
produced the same document; v1 output remained unchanged; omitting `--json` exited 2 with usage
on stderr. A two-difference pair produced both a `changed` and an `inserted` item in alignment
order. `view` with piped `N`/`q` still highlighted the inserted row and exited 0. All 36 tests
passed on Rust 1.100 with the GNU toolchain; the declared 1.83 minimum and physical terminals
remain untested.

## Stable context window (2026-11-22, third session)

`ui::render` now computes the window end first (`selected + 4`, capped by the row count) and
places the start at `end - 8`. Previously the start was `selected - 3`, so the last row showed
only four rows of context while a middle row showed eight. A pure renderer regression test
builds a 12-row comparison, selects the last row and asserts eight rendered rows with the cursor
on row 12 and row 5 still visible; it failed with four rows before the change. Rows near the
beginning keep their previous bounds. The running binary was re-checked with a 12-event pair
and piped navigation to the final row: eight rows render with row 5 still visible. All 37 tests
pass on Rust 1.100 with the GNU toolchain.
