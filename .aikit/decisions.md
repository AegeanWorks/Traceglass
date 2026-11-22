# Decisions

Why this project is built the way it is, oldest first. One entry per decision that changes a
contract, a dependency, a data format or the architecture; IDs are consecutive and never reused.

<!-- Entry format:
### ADR-NNN: <decision in one line>
- **Date**: YYYY-MM-DD
- **Status**: Accepted
- **Context**: <what forced the decision>
- **Decision**: <what was decided>
- **Consequences**: <what follows, including what it costs>
-->

### ADR-001: Trace comparison architecture
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: Developers need to find the first change between two execution traces without sending data to a service.
- **Decision**: Use a Rust-only local terminal interface, a deterministic pure trace comparison core, explicit navigation states, and file and console I/O in the shell.
- **Consequences**: Trace input uses a documented constrained format; snapshots are read once; the comparison cannot infer causal meaning.

### ADR-002: Align trace events by name with bounded edit distance
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: A positional mismatch makes later matching events look changed after an insertion and duplicates existing golden-trace checks.
- **Decision**: Provide deterministic minimum-cost alignment pairing equal names, with explicit changes, insertions and removals and a hard matrix-cell limit.
- **Consequences**: Repeated names may have several equally optimal alignments; deterministic tie rules select one and no causal inference is implied. Large comparisons are rejected rather than allocated without bound.

### ADR-003: Non-interactive comparison contract
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: Programmers need to use the same trace alignment in automated regression checks.
- **Decision**: Add check LEFT RIGHT with stable plain-text summary; exit zero for equal, one for differences, and two for usage or input errors.
- **Consequences**: The command reads each bounded snapshot once and uses the same alignment limit and tie rules as the interactive view; it does not launch a shell or mutate input files.

### ADR-004: Preserve native path arguments
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: Decoding every argument as Unicode panics on valid operating-system path values.
- **Decision**: Read operating-system arguments directly and pass file arguments as native paths; parse command and seed text explicitly.
- **Consequences**: Input file contents remain UTF-8; inaccessible or invalid native paths produce normal errors and the documented command exit codes.

### ADR-005: Expose aligned source positions
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: An aligned row number may not match either input file after an insertion or removal.
- **Decision**: Expose zero-based source event indices for each comparison row and render existing events with one-based input line numbers.
- **Consequences**: The trace format must continue to map one parsed event to one input line; absent sides have no source index. CI summary wording remains unchanged.

### ADR-006: Versioned JSON check summaries
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: Automation needs structured comparison results with source locations rather than parsing prose.
- **Decision**: Support check --json LEFT RIGHT with schema version 1, equality and row counts, and optional first difference with one-based source lines or null for absent sides.
- **Consequences**: The existing text format and exit codes stay unchanged; errors remain on stderr with no JSON result. The schema contains no trace values.

### ADR-007: Allow one standard-input trace in check mode
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: Programs should be able to pipe a trace directly into a non-interactive comparison.
- **Decision**: Treat one bare dash operand as standard input only for check; reject two dashes before reading and use the existing bounded UTF-8 reader.
- **Consequences**: A literal dash file in check is addressed as ./-; view still treats dash as a file because stdin is used for navigation. Exit codes and output formats are unchanged.

### ADR-008: Explicit raw line input mode
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: Ordinary programmer logs are not necessarily tab-separated event records.
- **Decision**: Add explicit --lines mode to view and check, parsing each UTF-8 line as an event named line using the existing bounded input adapter and alignment.
- **Consequences**: Structured traces remain the default; raw lines preserve blanks but normalize line terminators, and repeated lines can have ambiguous optimal alignments. JSON and piped check input remain supported.

### ADR-009: Bound the residual alignment matrix after a shared prefix
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: The full matrix limit rejects long traces even when they share a long identical beginning.
- **Decision**: Comparison matches identical leading events first, aligns the remaining slices with the existing matrix limit, and offsets source indices; the public full-matrix align function remains unchanged.
- **Consequences**: Long common-prefix traces can be compared without increasing the matrix limit. Early differences can still exceed it. Tests compare deterministic results against the full-matrix reference.

### ADR-010: Version 2 JSON reports every differing row
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: CI consumers of check --json only receive the first difference; repeated runs or prose parsing are needed to locate the rest, flags are order-sensitive, and there is no success exit for help
- **Decision**: Add check --all-differences (requires --json) emitting schema version 2 with an items array of every differing row, its one-based source lines and kind; parse flags in any order; add help/--help/-h; accept a 256-byte view command
- **Consequences**: Version 1 output and all exit codes stay byte-identical; --all-differences without --json is a usage error; kind is changed, inserted or removed; the view command limit now matches the documented 256 bytes

### ADR-011: Keep a full context window at the end of the view
- **Date**: 2026-11-22
- **Status**: Accepted
- **Context**: Anchoring the rendered window at selected-3 made the last rows show only four context rows, hiding earlier aligned rows exactly where a reviewer checks the final difference
- **Decision**: Derive the window end first, capped by the comparison length, and place the start eight rows before it; the cursor still selects the same row
- **Consequences**: Short and mid-list windows keep their previous bounds; near the end the view shows eight rows with the cursor at the bottom; navigation and source line rendering are unchanged
