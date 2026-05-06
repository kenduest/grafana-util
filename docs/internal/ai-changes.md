# ai-changes.md

Current AI change log only.

- Older detailed history moved to [`archive/ai-changes-archive-2026-03-24.md`](docs/internal/archive/ai-changes-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-changes-archive-2026-03-27.md`](docs/internal/archive/ai-changes-archive-2026-03-27.md).
- Detailed 2026-03-28 task notes were condensed into [`archive/ai-changes-archive-2026-03-28.md`](docs/internal/archive/ai-changes-archive-2026-03-28.md).
- Detailed 2026-03-29 through 2026-03-31 entries moved to [`archive/ai-changes-archive-2026-03-31.md`](docs/internal/archive/ai-changes-archive-2026-03-31.md).
- Detailed 2026-04-01 through 2026-04-12 entries moved to [`archive/ai-changes-archive-2026-04-12.md`](docs/internal/archive/ai-changes-archive-2026-04-12.md).
- Keep this file limited to the latest active architecture and maintenance changes.
- Older entries moved to [`ai-changes-archive-2026-04-13.md`](docs/internal/archive/ai-changes-archive-2026-04-13.md).
- Older entries moved to [`ai-changes-archive-2026-04-14.md`](docs/internal/archive/ai-changes-archive-2026-04-14.md).
- Older entries moved to [`ai-changes-archive-2026-04-15.md`](docs/internal/archive/ai-changes-archive-2026-04-15.md).
- Older entries moved to [`ai-changes-archive-2026-04-16.md`](docs/internal/archive/ai-changes-archive-2026-04-16.md).
- Older entries moved to [`ai-changes-archive-2026-04-17.md`](docs/internal/archive/ai-changes-archive-2026-04-17.md).
- Older entries moved to [`ai-changes-archive-2026-04-18.md`](docs/internal/archive/ai-changes-archive-2026-04-18.md).
- Older entries moved to [`ai-changes-archive-2026-04-19.md`](docs/internal/archive/ai-changes-archive-2026-04-19.md).
- Older entries moved to [`ai-changes-archive-2026-04-20.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-20.md).
- Older entries moved to [`ai-changes-archive-2026-04-26.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-26.md).
- Older entries moved to [`ai-changes-archive-2026-04-27.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-27.md).
- Older entries moved to [`ai-changes-archive-2026-04-28.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-28.md).
- Older entries moved to [`ai-changes-archive-2026-05-02.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-02.md).

## 2026-05-02 - Consume mutation review adapters
- Summary: added `build_review_mutation_summary_rows(&ReviewMutationEnvelope)` as a shared internal consumer for access import dry-run, datasource import dry-run, datasource live mutation, and alert plan review adapters.
- Tests: worker tests prove those adapters feed the shared summary consumer without public JSON drift.
- Impact: `rust/src/commands/review_contract.rs`, focused Rust tests, `todo.md`, and AI trace docs. Public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low internal-consumer change. Rollback would remove the shared summary projection while leaving public domain review and dry-run outputs unchanged.

## 2026-05-02 - Cleanup TODO trace after mutation adapter pass
- Summary: refreshed the active AI trace entry after the mutation adapter pass and kept this as docs/TODO cleanup only.
- Tests: no Rust tests were needed for this documentation-only update. Validation should be `make quality-ai-workflow` and `git diff --check`.
- Impact: `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`, and the current AI trace only. Rust behavior, public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low documentation-only change. Rollback would remove the refreshed TODO trace note and restore the previous active trace state.

## 2026-05-02 - Close remaining P3 TODO guardrail
- Summary: closed the remaining P3 TODO guardrail in the active AI trace and added a concrete backlog item to consume the review-adapter output on the next pass.
- Tests: no Rust tests were needed because this is docs/TODO cleanup only. Validation should be `make quality-ai-workflow` and `git diff --check`.
- Impact: `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`, and the current AI trace only. Rust behavior, public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low documentation-only change. Rollback would remove the guardrail closure note and the new review-adapter consumption backlog entry.

## 2026-05-02 - Extend mutation action adapters
- Summary: added internal `ReviewMutationAction` projections/envelopes for access import dry-run, datasource import dry-run, datasource live mutation preview, and alert plan rows without changing public JSON.
- Tests: followed TDD in worker slices for access, datasource import, datasource live mutation, and alert plan adapters; then ran focused adapter tests, broader access/datasource/alert filters, and full Rust validation.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet access_import_dry_run_review_envelope_projects_ready_and_blocked_rows`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_import_dry_run_review_projection_and_envelope_preserve_row_evidence`; `cargo test --manifest-path rust/Cargo.toml --quiet live_mutation_preview_review_projection_and_envelope_preserve_row_evidence`; `cargo test --manifest-path rust/Cargo.toml --quiet alert_rust_tests_runtime_plan::alert_plan_review_projection_maps_local_actions_without_changing_raw_rows`; `cargo test --manifest-path rust/Cargo.toml --quiet access`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource`; `cargo test --manifest-path rust/Cargo.toml --quiet alert`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/commands/access/import_dry_run.rs`, `rust/src/commands/datasource/import/dry_run.rs`, `rust/src/commands/datasource/mutation/render.rs`, `rust/src/commands/alert/runtime_support.rs`, `rust/src/commands/alert/tests/runtime_plan.rs`, `todo.md`, and AI trace docs. Public JSON, CLI behavior, `ReviewRisk`, `ReviewRequest`, legacy dashboard import dry-run, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low internal-adapter change. Rollback would remove normalized internal review views while leaving existing public dry-run/plan outputs unchanged; focused tests cover raw row preservation and action/status/blocked-reason mappings.
- Follow-up: keep `ReviewRisk` and `ReviewRequest` blocked until their cross-domain evidence improves; handle legacy dashboard import dry-run separately because its skip/fail actions need careful mapping.

## 2026-05-02 - Re-audit mutation review envelope evidence
- Summary: refreshed the mutation review envelope inventory with worker-backed evidence across dashboard/workspace, access/datasource, and alert/sync review surfaces.
- Tests: no Rust tests were needed because this is maintainer-only evidence and TODO routing. Validation covered AI workflow rules and whitespace checks.
- Test Run: `make quality-ai-workflow`; `git diff --check`.
- Impact: `docs/internal/mutation-review-envelope-inventory.md`, `todo.md`, and AI trace docs. Runtime behavior, public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low documentation-only checkpoint. Rollback would restore the previous ambiguous TODO state where blocked `ReviewRisk` / `ReviewRequest` work looked equally ready as mutation action adapter coverage.
- Follow-up: implement the next adapter pass for access import dry-run, datasource import dry-run, datasource live mutation preview, and alert plan rows only where the mapping is lossless and internal-only.

## 2026-05-02 - Reduce proven JSON clone hot spots
- Summary: removed avoidable `serde_json::Value`/object clones in owned Rust read and aggregation paths without changing public JSON or transport behavior.
- Tests: covered dashboard API response normalization, dashboard version timestamp lookup, sync live availability merge, request-backed contact-point availability extraction, live multi-org status aggregation, broader dashboard/sync_live/status filters, and full Rust validation.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_resource_client`; `cargo test --manifest-path rust/Cargo.toml --quiet latest_dashboard_version_timestamp`; `cargo test --manifest-path rust/Cargo.toml --quiet fetch_live_availability_with_request_collects_contact_points_and_plugins`; `cargo test --manifest-path rust/Cargo.toml --quiet merge_availability_deduplicates_arrays_and_overwrites_scalar_fields`; `cargo test --manifest-path rust/Cargo.toml --quiet build_live_multi_org_domain_status`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard`; `cargo test --manifest-path rust/Cargo.toml --quiet sync_live`; `cargo test --manifest-path rust/Cargo.toml --quiet status`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/grafana/api/dashboard.rs`, `rust/src/grafana/api/sync_live.rs`, `rust/src/grafana/api/sync_live_read.rs`, `rust/src/grafana/api/sync_live_read/availability.rs`, `rust/src/commands/status/live_multi_org.rs`, `rust/src/commands/sync/live_rust_tests.rs`, `todo.md`, and AI trace docs. Public JSON, CLI behavior, live transport semantics, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low behavior-preserving ownership cleanup. Rollback would restore cloning during response normalization and status aggregation; focused and broader Rust tests cover the unchanged output behavior.
- Follow-up: keep future `Value` clone cleanup evidence-led and avoid changing flexible JSON handling just to make data structures more static.

## 2026-04-28 - Normalize status producers
- Summary: converted document-backed staged/live alert, access, sync, and promotion status builders to domain-owned `StatusProducer` inputs while keeping live collection, read-failed fallback, multi-org merge, and transport-only placeholder rows outside the shared producer trait.
- Tests: covered staged alert, alert live, access live, staged sync, live sync, staged promotion, live promotion, broader `project_status`, and full Rust test suites.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet alert_project_status`; `cargo test --manifest-path rust/Cargo.toml --quiet alert_live_project_status`; `cargo test --manifest-path rust/Cargo.toml --quiet access::live_project_status::tests`; `cargo test --manifest-path rust/Cargo.toml --quiet sync_project_status_rust_tests`; `cargo test --manifest-path rust/Cargo.toml --quiet build_live_sync_domain_status`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status_promotion`; `cargo test --manifest-path rust/Cargo.toml --quiet live_project_status_promotion`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/commands/alert/project_status/staged.rs`, `rust/src/commands/alert/project_status/live.rs`, `rust/src/commands/access/live_project_status_build.rs`, `rust/src/commands/sync/project_status.rs`, `rust/src/commands/sync/live_project_status_sync.rs`, `rust/src/commands/sync/project_status_promotion.rs`, `rust/src/commands/sync/live_project_status_promotion.rs`, `todo.md`, and AI trace docs. Public JSON, generated docs, CLI behavior, live collection transport, and Python implementation are intentionally unchanged.
- Rollback/Risk: low structural refactor. Rollback would restore direct `StatusReading` construction in each domain builder; focused and full Rust tests cover the unchanged status output behavior.
- Follow-up: keep direct read-failed and transport-only fallback rows out of `StatusProducer` until they gain real domain-owned evidence.

## 2026-04-26 - Prove provisioning remains derived dashboard projection
- Summary: added regression coverage that provisioning dashboard files normalize to the same classic dashboard compare payload as raw export wrappers, and that sync bundle rejects explicit dual raw/provisioning dashboard inputs instead of treating provisioning as an alternate source of truth.
- Tests: covered raw wrapper vs provisioning compare normalization, provisioning classic payload shape, direct provisioning source loading, import dry-run provisioning roots, and sync bundle dual dashboard source rejection.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet compare_local_document_`; `cargo test --manifest-path rust/Cargo.toml --quiet source_loader_contract_resolves_direct_provisioning_root`; `cargo test --manifest-path rust/Cargo.toml --quiet collect_import_dry_run_report_accepts_provisioning_root_variant_metadata`; `cargo test --manifest-path rust/Cargo.toml --quiet run_sync_cli_bundle_rejects_dual_dashboard_sources`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`.
- Impact: `rust/src/commands/dashboard/import/compare.rs`, `rust/src/commands/sync/bundle_exec_sources_rust_tests.rs`, `todo.md`, and AI trace docs. Public JSON, generated docs, Python implementation, and dashboard v2 support are intentionally unchanged.
- Rollback/Risk: low test-only boundary hardening. Rollback would remove the regression coverage while leaving existing provisioning behavior unchanged.

## 2026-04-27 - Refresh dashboard directory re-layering inventory
- Summary: recorded a fresh maintainer-only dashboard re-layering inventory so future file moves are evidence-led rather than based on file size.
- Tests: no Rust tests were needed because this is docs/TODO inventory-only work. Workers ran targeted read-only inventory and one import/plan worker ran focused Rust filters while investigating.
- Test Run: `make quality-ai-workflow`; `git diff --check`.
- Impact: `docs/internal/dashboard-directory-relayering-inventory.md`, `todo.md`, and AI trace docs. No Rust code, public CLI/docs, generated docs, or runtime behavior changed.
- Rollback/Risk: low documentation-only checkpoint. Rollback would remove the fresh inventory and leave the future re-layering TODO without current evidence.

## 2026-04-27 - Move dashboard prompt transform boundary
- Summary: moved the shared dashboard prompt transform from root-level `prompt*.rs` modules into `dashboard/export_prompt/`, preserving facade re-exports for live export, raw-to-prompt, inspect, import validation, list metadata, and tests.
- Tests: covered raw-to-prompt conversion, prompt document generation, dashboard export/import inventory wiring, live library-panel prompt export support, and export-diff/test-support re-exports.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet build_external_export_document`; `cargo test --manifest-path rust/Cargo.toml --quiet raw_to_prompt`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_export_import_inventory_rust_tests`; `cargo test --manifest-path rust/Cargo.toml --quiet collect_library_panel_exports_with_request_records_failures_as_warnings`; `cargo test --manifest-path rust/Cargo.toml --quiet export_diff_rust_tests`; full Rust validation.
- Impact: `rust/src/commands/dashboard/export_prompt/`, dashboard facade/consumer imports, `todo.md`, and AI trace docs. Public CLI/docs, generated artifacts, Python implementation, and runtime behavior are intentionally unchanged.
- Rollback/Risk: low behavior-preserving module move. Rollback would restore the old root-level `prompt*.rs` layout; focused prompt/export tests cover the import wiring.
