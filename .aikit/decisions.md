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
