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

To compare ordinary UTF-8 logs, use `view --lines before.log after.log` or `check --lines before.log after.log`. Each line becomes an event named `line`, including blank lines; tabs remain part of the value. CRLF and LF line endings are treated alike, and a final line terminator does not add an extra event. Repeated lines can have multiple equally optimal alignments. `check --json --lines` supports structured output, and either input may still be piped with `-`.

For scripts and CI, run `cargo run --quiet -- check before.trace after.trace`. It prints a one-line summary to stdout and does not prompt. Exit status is `0` for equal traces, `1` for differences, and `2` for invalid usage, input, alignment limits, or output errors; diagnostics go to stderr. `view` retains its existing exit codes: `0` on success, `1` on runtime errors and `2` on invalid usage.

In `check`, use a single bare `-` in place of either file to read that trace from standard input, for example `producer | traceglass check expected.trace -`. The same 4 MiB and UTF-8 limits apply. Two `-` operands are rejected; use `./-` for a literal file named `-`. `view` treats `-` as a filename because its standard input controls navigation.

Use `check --json before.trace after.trace` for a versioned JSON result. Version 1 has `version`, `equal`, `rows`, `differences`, and `first`. When traces match, `first` is `null`; otherwise it contains the one-based aligned `row` and original `left_line` and `right_line`. A missing side is `null`. JSON mode keeps the same exit statuses; errors produce diagnostics on stderr and no JSON on stdout.

Alignment pairs events by name, so insertions do not necessarily shift every later match. Repeated names can produce multiple valid alignments; deterministic tie rules choose one. A difference does not establish its cause. Inputs are limited to 4 MiB each. Identical leading events are matched directly; the remaining alignment is limited to one million matrix cells, so long traces with early differences may still be rejected. Long event fields are clipped to 120 characters in the view.

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
