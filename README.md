# Traceglass

[![Built with AIKit](https://img.shields.io/badge/built%20with-AIKit-6f42c1)](https://github.com/AegeanWorks/AIKit)

Traceglass is a local Rust terminal workbench for comparing ordered execution traces.

## Cold-Start Quick Reference

- **Runtime Prerequisites**: Rust 1.83+ and Cargo with a working native linker. On Windows, use a GNU Rust toolchain with GCC or an MSVC toolchain with Visual C++ build tools. The current tests ran on Rust 1.100; the minimum has not yet been verified.
- **Canonical Regression Shield Gate**:
  ```bash
  cargo test
  ```

Each trace consists of UTF-8 lines containing an event name, a tab, and an event value. Run `cargo run -- view before.trace after.trace` to inspect a pair. Each event shows its original one-based file line; the number at the left of the row is its aligned position. Use `p` and `n` to move one row, `P` and `N` to jump between differences, and `q` to quit; press Enter after each command. Closing standard input also exits.

Try `cargo run -- view examples/before.trace examples/after.trace`, then enter `N` to jump to the inserted retry event. For a changed row, the view identifies the first differing character even when it lies beyond the clipped preview.

For scripts and CI, run `cargo run --quiet -- check before.trace after.trace`. It prints a one-line summary to stdout and reads no terminal input. Exit status is `0` for equal traces, `1` for differences, and `2` for invalid usage, input, alignment limits, or output errors; diagnostics go to stderr. `view` retains its existing exit codes: `0` on success, `1` on runtime errors and `2` on invalid usage.

Use `check --json before.trace after.trace` for a versioned JSON result. Version 1 has `version`, `equal`, `rows`, `differences`, and `first`. When traces match, `first` is `null`; otherwise it contains the one-based aligned `row` and original `left_line` and `right_line`. A missing side is `null`. JSON mode keeps the same exit statuses; errors produce diagnostics on stderr and no JSON on stdout.

Alignment pairs events by name, so insertions do not necessarily shift every later match. Repeated names can produce multiple valid alignments; deterministic tie rules choose one. A difference does not establish its cause. Inputs are limited to 4 MiB each and alignment to one million matrix cells; long event fields are clipped to 120 characters in the view.

File arguments use native operating-system paths, including non-Unicode filenames where the filesystem supports them. Trace file contents must still be UTF-8.

### What's Inside

```text
traceglass/
├── .aikit/       <- Project memory and delivery state
├── .gitattributes <- Git attributes
├── .gitignore    <- Build and transient state exclusions
├── Cargo.toml    <- Rust package manifest
├── Cargo.lock    <- Dependency lockfile
├── COMMERCIAL.md <- Commercial licensing terms
├── LICENSE       <- Personal license
├── LICENSES/     <- License text
├── README.md     <- Project guide
├── REUSE.toml    <- License metadata
├── assets/       <- Generated branding candidates
├── examples/     <- Example trace pair
├── reports/      <- AIKit observations and audit
├── src/          <- Rust core and adapters
└── tests/        <- Behavioral tests
```

## License

Free for personal use under the Aegean License 1.1; see [LICENSE](LICENSE). Commercial use requires the Aegean Commercial License; see [COMMERCIAL.md](COMMERCIAL.md).
