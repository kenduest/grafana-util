# ai-status-archive-2026-04-28

## 2026-04-26 - Add dashboard v2 adapter boundary regressions
- State: Done
- Scope: Rust dashboard diff/import source-wrapper regression tests, focused validation, and TODO trace. Public JSON, generated docs, classic dashboard behavior, and actual v2 adapter support are out of scope.
- Baseline: Classic raw/provisioning import and plan lanes already rejected dashboard v2 resources, but adapter-facing diff and root-export normalization paths still lacked dedicated regression coverage.
- Current Update: Added diff-lane tests proving raw and provisioning compare entrypoints reject dashboard v2 input before any remote compare request runs, and added import source-wrapper tests proving root export normalization into temp raw/provisioning variants still rejects v2 payloads.
- Result: Focused export-diff and import-loaded-source tests pass, and the remaining v2 adapter-boundary TODO is now satisfied.
