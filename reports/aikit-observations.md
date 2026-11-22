# AIKit observations during Traceglass setup

## 2026-11-22

- `aikit new` created a separate Git repository and task memory, then left local brand candidates untracked. The project ledger and guidelines provide a durable task and architectural contract.
- The new software scaffold included `demo.py` and `inventory.py` even though this project is Rust-only. These are generic placeholders, not a Rust implementation. A stack-aware scaffold would avoid generating incompatible source files.
- `ui init` reported UI kind `web` for a manually declared terminal surface. The project UI identifier was corrected with `ui-id --set yes --kind cli`. The UI contract must be committed before `ui begin`, including when recovering with `--since HEAD`.
- The AIKit root reports inbox already exceeded its stated cap at startup. This project retains its requested observations here, separately from that local-only inbox.
- The default MSVC toolchain could not find `link.exe`; the already-installed GNU toolchain and GCC successfully ran the two initial Rust tests. This is a host setup issue, not an AIKit defect.
- The first project gate failed hygiene as inconclusive because it scans Python and web assets, not Rust, and failed compatibility because the scaffold's commands invoke its Python placeholders. This prevents a Rust-only project from passing the default project gates without a supported adaptation. The documentation check also requires specific cold-start and tree headings; that finding is project documentation work.
- The built-in hygiene gate cannot be replaced through project gate overrides (`scripts/project_gates.py`); adding a dummy scannable file would manufacture evidence rather than check the Rust code. The compatibility record can declare non-Python commands, but those commands must exercise real program behavior and deterministic replay.
- On the next session, `sync_principles.py --involved` refused because the AIKit native DLL was older than its Rust sources. Its diagnostic recommends rebuilding the native library; until then the involved-principle lookup was not available. This is a freshness refusal, not evidence of a defect in the parser itself.
- Concurrent uncommitted changes in the AIKit checkout appear to add Rust source hygiene scanning. They belong to another workstream and were not modified or relied upon here; their completion and verification remain unknown.
- AIKit commit `df4bab81` subsequently added Rust text hygiene support; the Traceglass hygiene gate now passes. Its scope is text hygiene, not a proof of Rust semantics. Traceglass replaced the scaffold compatibility commands with a Rust inventory and deterministic scripted comparison; AIKit's compatibility gate passes against those commands.
