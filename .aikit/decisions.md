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
