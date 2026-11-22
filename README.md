# Traceglass

[![Built with AIKit](https://img.shields.io/badge/built%20with-AIKit-6f42c1)](https://github.com/AegeanWorks/AIKit)

Traceglass is a local Rust tool under development for comparing ordered execution traces.

## Cold-Start Quick Reference

- **Runtime Prerequisites**: Rust 1.70+ and Cargo with a working native linker. On Windows, use a GNU Rust toolchain with GCC or an MSVC toolchain with Visual C++ build tools. The current tests ran on Rust 1.100; the minimum has not yet been verified.
- **Canonical Regression Shield Gate**:
  ```bash
  cargo test
  ```

Each trace consists of UTF-8 lines containing an event name, a tab, and an event value. The core supports positional comparison and bounded alignment by event name, so insertions do not necessarily shift every later match. Repeated names can produce multiple valid alignments; deterministic tie rules choose one. A difference does not establish its cause. The interactive interface is still under development.

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
├── reports/      <- AIKit observations and audit
├── src/          <- Rust core and adapters
└── tests/        <- Behavioral tests
```

## License

Free for personal use under the Aegean License 1.1; see [LICENSE](LICENSE). Commercial use requires the Aegean Commercial License; see [COMMERCIAL.md](COMMERCIAL.md).
