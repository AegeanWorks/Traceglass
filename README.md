# Traceglass

[![Built with AIKit](https://img.shields.io/badge/built%20with-AIKit-6f42c1)](https://github.com/AegeanWorks/AIKit)

A local terminal workbench for comparing structured program execution traces and explaining the first divergence.

## ⚡ 30-Second Cold-Start Quick Reference

- **Runtime Prerequisites**: <FILL: each runtime this project needs and its minimum version>
- **Canonical Regression Shield Gate**:
  ```bash
  <FILL: the command that runs the whole test suite from this project root>
  ```

### 🛠️ AIKit Commands (run from the AIKit folder)
| Role | Command |
|---|---|
| Status, task and gates | `python aikit.py status traceglass` |
| Phase 4: every gate | `python aikit.py check traceglass` |
| Repair memory and map, then check | `python aikit.py fix traceglass` |
| Resume an interrupted task | `python aikit.py resume traceglass` |

## 📦 What's Inside

```text
traceglass/
├── .gitattributes            <- LF line endings on every platform
├── .gitignore                <- transient AIKit state excluded
├── LICENSE                   <- Aegean License 1.1 (AL-1.1): free personal use
├── COMMERCIAL.md             <- Commercial licensing terms
├── LICENSES/                 <- REUSE license texts
├── REUSE.toml                <- Machine-readable SPDX metadata
├── README.md
├── inventory.py              <- AI-compat inventory: kind|name|locator lines (ADR-315)
├── demo.py                   <- AI-compat scripted drive: WORLD-DELTA lines (ADR-315)
└── .aikit/                   <- this project's memory: record, ledger, decisions
```

## 📜 License

Free for personal use under the Aegean License 1.1 (AL-1.1); see [LICENSE](LICENSE). Any commercial use needs the Aegean Commercial License 1.0 (ACL-1.0); see [COMMERCIAL.md](COMMERCIAL.md).
