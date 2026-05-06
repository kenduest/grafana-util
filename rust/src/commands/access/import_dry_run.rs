//! Shared access import dry-run review document builder.

use serde_json::{Map, Value};
use std::path::Path;

use crate::common::tool_version;
use crate::review_contract::{
    build_review_mutation_envelope, ReviewBlockedReason, ReviewMutationAction,
    ReviewMutationActionInput, ReviewMutationEnvelope, REVIEW_ACTION_BLOCKED,
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_UPDATE, REVIEW_STATUS_BLOCKED,
    REVIEW_STATUS_READY,
};

const ACCESS_IMPORT_DRY_RUN_KIND: &str = "grafana-utils-access-import-dry-run";
const ACCESS_IMPORT_DRY_RUN_SCHEMA_VERSION: i64 = 1;
const ACCESS_IMPORT_DRY_RUN_DOMAIN: &str = "access";

pub(crate) fn build_access_import_dry_run_document(
    resource_kind: &str,
    rows: &[Map<String, Value>],
    processed: usize,
    created: usize,
    updated: usize,
    skipped: usize,
    source: &Path,
) -> Value {
    let blocked = rows
        .iter()
        .filter(|row| {
            matches!(row.get("blocked"), Some(Value::Bool(true)))
                || matches!(row.get("status"), Some(Value::String(status)) if status == "blocked")
        })
        .count();
    Value::Object(Map::from_iter(vec![
        (
            "kind".to_string(),
            Value::String(ACCESS_IMPORT_DRY_RUN_KIND.to_string()),
        ),
        (
            "schemaVersion".to_string(),
            Value::Number(ACCESS_IMPORT_DRY_RUN_SCHEMA_VERSION.into()),
        ),
        (
            "toolVersion".to_string(),
            Value::String(tool_version().to_string()),
        ),
        ("reviewRequired".to_string(), Value::Bool(true)),
        ("reviewed".to_string(), Value::Bool(false)),
        (
            "resourceKind".to_string(),
            Value::String(resource_kind.to_string()),
        ),
        (
            "rows".to_string(),
            Value::Array(rows.iter().cloned().map(Value::Object).collect()),
        ),
        (
            "summary".to_string(),
            Value::Object(Map::from_iter(vec![
                (
                    "processed".to_string(),
                    Value::Number((processed as i64).into()),
                ),
                (
                    "created".to_string(),
                    Value::Number((created as i64).into()),
                ),
                (
                    "updated".to_string(),
                    Value::Number((updated as i64).into()),
                ),
                (
                    "skipped".to_string(),
                    Value::Number((skipped as i64).into()),
                ),
                (
                    "blocked".to_string(),
                    Value::Number((blocked as i64).into()),
                ),
                (
                    "source".to_string(),
                    Value::String(source.to_string_lossy().to_string()),
                ),
            ])),
        ),
    ]))
}

pub(crate) type AccessImportDryRunReviewActionProjection = ReviewMutationAction;

#[derive(Debug, Clone)]
pub(crate) struct AccessImportDryRunReviewProjection {
    pub domains: Vec<&'static str>,
    pub actions: Vec<ReviewMutationAction>,
}

fn access_import_dry_run_action_id(resource_kind: &str, row: &Map<String, Value>) -> String {
    let identity = row
        .get("identity")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(resource_kind);
    let index = row
        .get("index")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("row");
    format!("access-import-dry-run:{resource_kind}:{index}:{identity}")
}

fn access_import_dry_run_status(row: &Map<String, Value>) -> String {
    if matches!(row.get("blocked"), Some(Value::Bool(true)))
        || matches!(row.get("status"), Some(Value::String(status)) if status == REVIEW_STATUS_BLOCKED)
    {
        REVIEW_STATUS_BLOCKED.to_string()
    } else {
        match row.get("status").and_then(Value::as_str).map(str::trim) {
            Some(REVIEW_STATUS_READY) => REVIEW_STATUS_READY.to_string(),
            Some("same") => "same".to_string(),
            Some("warning") => "warning".to_string(),
            _ => REVIEW_STATUS_READY.to_string(),
        }
    }
}

fn access_import_dry_run_action(row: &Map<String, Value>, status: &str) -> String {
    if status == REVIEW_STATUS_BLOCKED {
        return REVIEW_ACTION_BLOCKED.to_string();
    }
    match row.get("action").and_then(Value::as_str).map(str::trim) {
        Some("created" | "create") => REVIEW_ACTION_WOULD_CREATE.to_string(),
        Some(_) => REVIEW_ACTION_WOULD_UPDATE.to_string(),
        None => REVIEW_ACTION_WOULD_UPDATE.to_string(),
    }
}

fn access_import_dry_run_identity(resource_kind: &str, row: &Map<String, Value>) -> String {
    row.get("identity")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| resource_kind.to_string())
}

fn access_import_dry_run_details(row: &Map<String, Value>) -> Option<String> {
    row.get("detail")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn access_import_dry_run_blocked_reason(row: &Map<String, Value>) -> Option<String> {
    ReviewBlockedReason::from_optional_text(
        row.get("blockedReason")
            .and_then(Value::as_str)
            .or_else(|| row.get("blocked_reason").and_then(Value::as_str)),
    )
    .or_else(|| {
        row.get("blockers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::trim)
            .find(|value| !value.is_empty())
            .and_then(ReviewBlockedReason::from_text)
    })
    .map(ReviewBlockedReason::into_string)
}

fn access_import_dry_run_review_hints(row: &Map<String, Value>) -> Vec<String> {
    row.get("reviewHints")
        .or_else(|| row.get("review_hints"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn access_import_dry_run_review_action(
    resource_kind: &str,
    row: &Map<String, Value>,
) -> AccessImportDryRunReviewActionProjection {
    let status = access_import_dry_run_status(row);
    ReviewMutationActionInput {
        action_id: access_import_dry_run_action_id(resource_kind, row),
        action: access_import_dry_run_action(row, &status),
        domain: ACCESS_IMPORT_DRY_RUN_DOMAIN.to_string(),
        resource_kind: resource_kind.to_string(),
        identity: access_import_dry_run_identity(resource_kind, row),
        status,
        blocked_reason: access_import_dry_run_blocked_reason(row),
        details: access_import_dry_run_details(row),
        review_hints: access_import_dry_run_review_hints(row),
        raw: Value::Object(row.clone()),
    }
    .into()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_access_import_dry_run_review_projection(
    document: &Value,
) -> Option<AccessImportDryRunReviewProjection> {
    let object = document.as_object()?;
    if object.get("kind").and_then(Value::as_str) != Some(ACCESS_IMPORT_DRY_RUN_KIND) {
        return None;
    }
    let resource_kind = object.get("resourceKind").and_then(Value::as_str)?;
    let rows = object.get("rows").and_then(Value::as_array)?;
    Some(AccessImportDryRunReviewProjection {
        domains: vec![ACCESS_IMPORT_DRY_RUN_DOMAIN],
        actions: rows
            .iter()
            .filter_map(Value::as_object)
            .map(|row| access_import_dry_run_review_action(resource_kind, row))
            .collect(),
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_access_import_dry_run_review_envelope(
    document: &Value,
) -> Option<ReviewMutationEnvelope> {
    let projection = build_access_import_dry_run_review_projection(document)?;
    Some(build_review_mutation_envelope(
        projection.actions,
        &projection.domains,
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        build_access_import_dry_run_document, build_access_import_dry_run_review_envelope,
    };
    use crate::review_contract::build_review_mutation_summary_rows;
    use serde_json::{json, Map, Value};
    use std::path::Path;

    #[test]
    fn access_import_dry_run_review_envelope_projects_ready_and_blocked_rows() {
        let ready_row = Map::from_iter(vec![
            ("index".to_string(), json!("1")),
            ("identity".to_string(), json!("alice@example.com")),
            ("action".to_string(), json!("created")),
            ("status".to_string(), json!("ready")),
            ("blocked".to_string(), json!(false)),
            (
                "detail".to_string(),
                json!("would create user alice@example.com"),
            ),
            (
                "reviewHints".to_string(),
                json!(["review org role assignment"]),
            ),
            (
                "target".to_string(),
                json!({
                    "targetId": "12",
                    "email": "alice@example.com"
                }),
            ),
        ]);
        let blocked_row = Map::from_iter(vec![
            ("index".to_string(), json!("2")),
            ("identity".to_string(), json!("ops")),
            ("action".to_string(), json!("updated")),
            ("status".to_string(), json!("blocked")),
            ("blocked".to_string(), json!(true)),
            (
                "detail".to_string(),
                json!("provisioned team memberships cannot be changed"),
            ),
            (
                "blockers".to_string(),
                json!(["provisioned team memberships cannot be changed"]),
            ),
            (
                "reviewHints".to_string(),
                json!(["review the provisioned team target before changing membership"]),
            ),
            (
                "target".to_string(),
                json!({
                    "targetId": "44",
                    "targetUid": "team-ops",
                    "isProvisioned": true
                }),
            ),
        ]);
        let rows = vec![ready_row.clone(), blocked_row.clone()];

        let document = build_access_import_dry_run_document(
            "user",
            &rows,
            2,
            1,
            1,
            0,
            Path::new("/tmp/access-users"),
        );

        let review = build_access_import_dry_run_review_envelope(&document).unwrap();

        assert_eq!(review.actions.len(), 2);
        assert_eq!(review.domains.len(), 1);
        assert_eq!(review.domains[0].id, "access");
        assert_eq!(review.summary.action_count, 2);
        assert_eq!(review.summary.blocked_count, 1);

        let ready = &review.actions[0];
        assert_eq!(ready.domain, "access");
        assert_eq!(ready.identity, "alice@example.com");
        assert_eq!(ready.action, "would-create");
        assert_eq!(ready.status, "ready");
        assert_eq!(
            ready.details.as_deref(),
            Some("would create user alice@example.com")
        );
        assert_eq!(
            ready.review_hints,
            vec!["review org role assignment".to_string()]
        );
        assert_eq!(ready.raw, Value::Object(ready_row));

        let blocked = &review.actions[1];
        assert_eq!(blocked.domain, "access");
        assert_eq!(blocked.identity, "ops");
        assert_eq!(blocked.action, "blocked");
        assert_eq!(blocked.status, "blocked");
        assert_eq!(
            blocked.blocked_reason.as_deref(),
            Some("provisioned team memberships cannot be changed")
        );
        assert_eq!(
            blocked.details.as_deref(),
            Some("provisioned team memberships cannot be changed")
        );
        assert_eq!(
            blocked.review_hints,
            vec!["review the provisioned team target before changing membership".to_string()]
        );
        assert_eq!(blocked.raw, Value::Object(blocked_row));
        assert_eq!(
            review.blocked_reasons,
            vec!["provisioned team memberships cannot be changed".to_string()]
        );

        assert_eq!(
            document.get("rows").and_then(Value::as_array),
            Some(&vec![
                Value::Object(ready.raw.as_object().cloned().unwrap()),
                Value::Object(blocked.raw.as_object().cloned().unwrap()),
            ])
        );
    }

    #[test]
    fn access_import_dry_run_review_envelope_feeds_shared_summary_rows_without_public_json_drift() {
        let ready_row = Map::from_iter(vec![
            ("index".to_string(), json!("1")),
            ("identity".to_string(), json!("alice@example.com")),
            ("action".to_string(), json!("created")),
            ("status".to_string(), json!("ready")),
            ("blocked".to_string(), json!(false)),
            (
                "detail".to_string(),
                json!("would create user alice@example.com"),
            ),
        ]);
        let rows = vec![ready_row.clone()];
        let document = build_access_import_dry_run_document(
            "user",
            &rows,
            1,
            1,
            0,
            0,
            Path::new("/tmp/access-users"),
        );
        let public_rows_before = document["rows"].clone();

        let review = build_access_import_dry_run_review_envelope(&document).unwrap();
        let summary_rows = build_review_mutation_summary_rows(&review);

        assert_eq!(summary_rows.len(), 1);
        assert_eq!(summary_rows[0].domain, "access");
        assert_eq!(summary_rows[0].resource_kind, "user");
        assert_eq!(summary_rows[0].identity, "alice@example.com");
        assert_eq!(summary_rows[0].action, "would-create");
        assert_eq!(summary_rows[0].status, "ready");
        assert_eq!(
            summary_rows[0].details.as_deref(),
            Some("would create user alice@example.com")
        );
        assert_eq!(summary_rows[0].action_count, 1);
        assert_eq!(summary_rows[0].domain_count, 1);
        assert_eq!(summary_rows[0].blocked_count, 0);
        assert_eq!(summary_rows[0].warning_count, 0);
        assert!(summary_rows[0].blocked_reasons.is_empty());
        assert_eq!(document["rows"], public_rows_before);
        assert_eq!(document["rows"][0], Value::Object(ready_row));
    }
}
