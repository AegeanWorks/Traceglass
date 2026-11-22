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

## Portfolio comparison

The public repository inventory was retrieved through GitHub's API on 2026-11-22 (one page of 100 was sufficient; page two was empty). Sextant already verifies state machines and checks golden traces for the first differing line. Traceglass compares generic program event traces and offers interactive inspection of bounded insertion/deletion-aware alignment. This distinguishes its workflow from Sextant; it is not a claim of global novelty.

The profile's paginated starred-repository names were also inspected. Descriptions of additional projects not present in the public owner inventory did not identify another trace comparison workbench.
