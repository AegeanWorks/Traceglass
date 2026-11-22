# Architecture audit — work in progress

## 2026-11-22

- `parse_trace`, `compare`, `align`, `Comparison`, and `Navigation::apply` operate on values and perform no I/O. The positional comparison shifts after an insertion; bounded minimum-cost alignment can preserve later matches, and navigation can use aligned rows. Neither mode proves causality.
- `input::read_trace` is an edge adapter: it caps bytes before decoding, rejects invalid UTF-8 and returns parsing errors rather than writing output. The eventual shell must open files and display errors.
- Navigation is an explicit `Browsing` or `Finished` state. Empty inputs and replacing the comparison with a shorter snapshot are covered by tests; `Finished` ignores later actions.
- The shell, interactive terminal surface, real compatibility demo, error presentation, and end-to-end tests are not implemented yet. No claim of a complete architectural audit or usable application is warranted.

## Portfolio comparison

The public repository inventory was retrieved through GitHub's API on 2026-11-22 (one page of 100 was sufficient; page two was empty). Sextant already verifies state machines and checks golden traces for the first differing line. Traceglass targets generic program event traces and interactive inspection of bounded insertion/deletion-aware alignment. This distinguishes its intended workflow from Sextant; it is not a claim of global novelty or a completed user experience.
