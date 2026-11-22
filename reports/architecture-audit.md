# Architecture audit — work in progress

## 2026-11-22

- `parse_trace`, `compare`, `align`, `Comparison`, and `Navigation::apply` operate on values and perform no I/O. The positional comparison shifts after an insertion; bounded minimum-cost alignment can preserve later matches, and navigation can use aligned rows. Neither mode proves causality.
- `input::read_trace` is an edge adapter: it caps bytes before decoding, rejects invalid UTF-8 and returns parsing errors rather than writing output. The shell opens files and displays errors.
- Navigation is an explicit `Browsing` or `Finished` state. Empty inputs and replacing the comparison with a shorter snapshot are covered by tests; `Finished` ignores later actions.
- `src/main.rs` owns file and console I/O; it reads each file once, bounds inputs and alignment work, handles EOF, and maps commands into pure navigation actions. The terminal view escapes control and bidirectional formatting characters and clips long fields. The Rust compatibility demo uses the same alignment core.
- A scripted end-to-end test covers two files and navigation. On 2026-11-22 the agent also ran the interface with an inserted event, inspected the rendered rows, and confirmed `N` selected the insertion and `q` exited. A physical terminal, other operating systems, and the declared minimum Rust version have not yet been tested.
- After forcing Cargo to recompile restored files, a second terminal run displayed the first difference at character 131 in clipped values; the corresponding regression test passed. Alignment remains heuristic for repeated names and is bounded to one million cells.
- A fresh Cargo target directory was used for `cargo test --locked`; all 17 tests passed on the Windows GNU toolchain. This is independent of the previous target's timestamp-based cache.

## Portfolio comparison

The public repository inventory was retrieved through GitHub's API on 2026-11-22 (one page of 100 was sufficient; page two was empty). Sextant already verifies state machines and checks golden traces for the first differing line. Traceglass targets generic program event traces and interactive inspection of bounded insertion/deletion-aware alignment. This distinguishes its intended workflow from Sextant; it is not a claim of global novelty or a completed user experience.
