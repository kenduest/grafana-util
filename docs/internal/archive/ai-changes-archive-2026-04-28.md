# ai-changes-archive-2026-04-28

## 2026-04-20 - Add dashboard folder permission drift review
- Summary: added a read-only folder permission drift lane to `dashboard plan`, including `--include-folder-permissions`, UID-first matching, optional `uid-then-path` fallback, folder permission action rows, permission detail rendering, and synced English/zh-TW command docs.
- Tests: added parser/help coverage, permission drift action coverage for same/update/extra/missing/path-fallback cases, and an input-collection regression that loads `raw/permissions.json` and fetches live folder permissions.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_parser_help_workflow --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-docs-surface`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/commands/dashboard/cli_defs_command_plan.rs`, `rust/src/commands/dashboard/plan/`, `rust/src/commands/dashboard/plan_types.rs`, `rust/src/commands/dashboard/dashboard_runtime.rs`, `docs/commands/en/dashboard-plan.md`, `docs/commands/zh-TW/dashboard-plan.md`, and AI trace docs. Import-time permission restore, dashboard ACL diff, Python implementation, and generated docs are intentionally unchanged.
- Rollback/Risk: moderate review-surface change. Rollback would remove the optional flag and permission action lane while keeping existing dashboard plan behavior; the feature is opt-in and read-only, but plan summary counts include folder permission rows when enabled.
- Follow-up: add dashboard permission diff or import-time folder permission restore only after subject-resolution and ACL apply policy are finalized.
