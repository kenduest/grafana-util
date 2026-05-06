# Developer Guide

This page is the maintainer landing page for the repo.

Use it to orient quickly, decide which lane you are in, and jump to the
document or code surface that actually owns the change. Do not turn this file
into a second spec, a second quickstart, or a long policy dump.

## Start Here

If you just entered the repo:

1. `README.md`
   - public product shape and operator-facing entrypoints
2. `docs/internal/maintainer-quickstart.md`
   - first-entry maintainer routing
3. `rust/src/cli/mod.rs`
   - public CLI topology and namespace wiring
4. `docs/internal/maintainer-role-map.md`
   - routing by maintainer persona
5. `docs/internal/contract-doc-map.md`
   - where stable contracts and policy docs live
6. `Makefile`
   - supported validation and generation entrypoints

If you are returning to the repo after time away, start with
`docs/internal/maintainer-quickstart.md` first and treat this page as the short
map, not the full walkthrough.

## Repo Priorities

- maintained implementation surface:
  - `rust/src/`
- legacy reference surface:
  - `python/grafana_utils/`
- public operator docs:
  - `README.md`, `README.zh-TW.md`, `docs/user-guide/`
- command-reference source:
  - `docs/commands/`
- generated artifacts:
  - `docs/man/`, `docs/html/`

Rust is the supported product surface. Python is legacy reference material
unless the task explicitly targets the Python lane.

## Current Public CLI Shape

Treat these namespaces as the maintained public surface:

- `grafana-util dashboard`
- `grafana-util alert`
- `grafana-util access`
- `grafana-util status`
- `grafana-util workspace`
- `grafana-util export`
- `grafana-util config`

For command topology and help routing, start with `rust/src/cli/mod.rs`. For help
rendering specifically, also check `rust/src/cli/help/mod.rs`.

## Choose Your Lane

If the task is mostly:

- runtime behavior, flags, parser rules, help, or dispatch:
  - start with `rust/src/cli/mod.rs` and the owning Rust module under `rust/src/`
- dashboard behavior:
  - start with `rust/src/commands/dashboard/`
- datasource behavior:
  - start with `rust/src/commands/datasource/mod.rs`
- alert behavior:
  - start with `rust/src/commands/alert/mod.rs`
- access behavior:
  - start with `rust/src/commands/access/`
- status, snapshot, resource, or workspace/change runtime behavior:
  - start with `rust/src/commands/sync/`
- handbook or command docs:
  - start with `README.md`, `README.zh-TW.md`, `docs/user-guide/`, and `docs/commands/`
- generated HTML or manpages:
  - start with `docs/internal/generated-docs-architecture.md`,
    `docs/internal/generated-docs-playbook.md`, and the generator scripts under
    `scripts/`
- contract or schema meaning:
  - start with `docs/internal/contract-doc-map.md`
- secret storage or profile behavior:
  - start with `docs/internal/profile-secret-storage-architecture.md`
- build, packaging, install, or release workflow:
  - start with `Makefile`, `scripts/`, `rust/Cargo.toml`, and `python/pyproject.toml`
- AI-assisted maintainer workflow:
  - start with `docs/internal/ai-workflow-note.md` and
    `docs/internal/ai-change-closure-rules.md`

If you prefer role-based routing instead of subsystem routing, use
`docs/internal/maintainer-role-map.md`.

## Documentation Boundaries

Keep the doc families separate:

- `README.md`, `README.zh-TW.md`, `docs/user-guide/`
  - workflow, intent, examples, recommended reading order
- `docs/commands/`
  - per-command reference content and exact syntax
- `docs/man/`, `docs/html/`
  - generated artifacts, not primary source
- `docs/internal/*.md`
  - maintainer contracts, policy, architecture, and workflow notes
- `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`
  - trace and condensed history, not long-form design docs

If a change starts crossing those boundaries, use
`docs/internal/docs-architecture-guardrails.md`.

## Validation Defaults

Prefer repo-owned commands over ad hoc one-offs.

Common entrypoints:

```bash
# Purpose: Common maintainer validation entrypoints.
make help
make test
make test-rust
make test-python
make man-check
make html-check
make quality-docs-surface
make quality-ai-workflow
```

For generated docs work:

```bash
# Purpose: Regenerate derived docs artifacts.
make man
make html
```

## macOS To Linux amd64 Builds

For macOS developers building the native Linux `amd64` Rust artifact without
Docker, prepare the user-global toolchain first:

```bash
# Purpose: Install the non-Docker Linux amd64 build tools on macOS.
brew install zig rustup
export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"
/opt/homebrew/opt/rustup/bin/rustup default stable
/opt/homebrew/opt/rustup/bin/rustup target add x86_64-unknown-linux-gnu
cargo install cargo-zigbuild
```

Homebrew `rustup` is keg-only because it conflicts with Homebrew `rust`. Put
`/opt/homebrew/opt/rustup/bin` before `/opt/homebrew/bin` when running cross
builds so `cargo` and `rustc` come from rustup. For future shells, add:

```bash
# Purpose: Prefer rustup-managed Rust commands for cross builds.
export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"
```

Then run:

```bash
# Purpose: Build the default Linux amd64 Rust artifact.
make build-rust-linux-amd64
```

The zig build script also prefers LLVM `llvm-ar` for Linux archives. If the
local zig path fails or you want an isolated toolchain, use the Docker fallback:

```bash
# Purpose: Build the Linux amd64 Rust artifact in Docker.
make build-rust-linux-amd64-docker
```

## Maintenance Rules

- Update Rust behavior and help text first; treat Python as legacy unless the
  task explicitly needs parity work there.
- Keep public CLI/docs routing in sync with
  `scripts/contracts/command-surface.json`.
- Keep generated docs derived from source; do not patch `docs/html/` or
  `docs/man/` as if they were canonical.
- Keep contract detail in the dedicated internal policy/spec docs, not here.
- Keep facades thin: `cli.rs` and domain `mod.rs` files should route and
  normalize, not absorb downstream contract logic.
- Keep handbook/manual content and command-reference content separate.
- Keep comments high-signal: explain ownership, invariants, and non-obvious
  behavior rather than narrating control flow.

## When To Update Other Maintainer Docs

When you make a meaningful architecture, contract, or workflow change:

- update the owning narrow spec or internal doc first
- update this page only if maintainer entry routing changed
- update `docs/internal/ai-status.md` and `docs/internal/ai-changes.md` only
  when the change is meaningful enough to trace

For the current summary/spec/trace split, start with
[`docs/internal/contract-doc-map.md`](docs/internal/contract-doc-map.md).
