//! Renderers for datasource mutation dry-run and import result payloads.
//!
//! Responsibilities:
//! - Convert mutation rows into structured table/json output.
//! - Validate dry-run arguments for command-line consistency.
//! - Provide shared formatting between `mutation` and `import` workflows.

use serde_json::{Map, Value};

use crate::common::{message, requested_columns_include_all, Result};
use crate::review_contract::{
    build_review_mutation_envelope, review_action_rank, ReviewMutationAction,
    ReviewMutationActionInput, ReviewMutationEnvelope, REVIEW_ACTION_BLOCKED,
    REVIEW_ACTION_BLOCKED_AMBIGUOUS, REVIEW_ACTION_BLOCKED_UID_MISMATCH,
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE,
    REVIEW_REASON_AMBIGUOUS_LIVE_NAME_MATCH, REVIEW_REASON_UID_NAME_MISMATCH,
    REVIEW_STATUS_BLOCKED, REVIEW_STATUS_READY,
};

#[derive(Debug, Clone)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DatasourceLiveMutationReviewProjection {
    pub(crate) domains: Vec<&'static str>,
    pub(crate) actions: Vec<ReviewMutationAction>,
}

fn mutation_row_value(row: &[String], index: usize) -> &str {
    row.get(index).map(String::as_str).unwrap_or("")
}

fn build_live_mutation_row_raw(row: &[String]) -> Value {
    Value::Object(Map::from_iter(vec![
        (
            "operation".to_string(),
            Value::String(mutation_row_value(row, 0).to_string()),
        ),
        (
            "uid".to_string(),
            Value::String(mutation_row_value(row, 1).to_string()),
        ),
        (
            "name".to_string(),
            Value::String(mutation_row_value(row, 2).to_string()),
        ),
        (
            "type".to_string(),
            Value::String(mutation_row_value(row, 3).to_string()),
        ),
        (
            "match".to_string(),
            Value::String(mutation_row_value(row, 4).to_string()),
        ),
        (
            "action".to_string(),
            Value::String(mutation_row_value(row, 5).to_string()),
        ),
        (
            "targetId".to_string(),
            Value::String(mutation_row_value(row, 6).to_string()),
        ),
    ]))
}

fn live_mutation_review_identity(row: &[String]) -> String {
    let uid = mutation_row_value(row, 1).trim();
    if !uid.is_empty() {
        return uid.to_string();
    }
    let name = mutation_row_value(row, 2).trim();
    if !name.is_empty() {
        return name.to_string();
    }
    let operation = mutation_row_value(row, 0).trim();
    if !operation.is_empty() {
        return operation.to_string();
    }
    "unknown".to_string()
}

fn live_mutation_review_action_id(row: &[String], identity: &str) -> String {
    let operation = mutation_row_value(row, 0).trim();
    let target_id = mutation_row_value(row, 6).trim();
    let identity_kind = if mutation_row_value(row, 1).trim().is_empty() {
        "identity"
    } else {
        "uid"
    };
    format!(
        "datasource-live-mutation:{}:{}:{}:target:{}",
        if operation.is_empty() {
            "unknown"
        } else {
            operation
        },
        identity_kind,
        identity,
        if target_id.is_empty() {
            "none"
        } else {
            target_id
        }
    )
}

fn live_mutation_review_details(row: &[String]) -> Option<String> {
    let mut parts = vec![
        format!("operation={}", mutation_row_value(row, 0)),
        format!("match={}", mutation_row_value(row, 4)),
    ];
    if !mutation_row_value(row, 6).trim().is_empty() {
        parts.push(format!("targetId={}", mutation_row_value(row, 6)));
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

fn normalize_live_mutation_review_action(
    row: &[String],
) -> (&'static str, &'static str, Option<String>) {
    match mutation_row_value(row, 5) {
        "would-create" => (REVIEW_ACTION_WOULD_CREATE, REVIEW_STATUS_READY, None),
        "would-update" => (REVIEW_ACTION_WOULD_UPDATE, REVIEW_STATUS_READY, None),
        "would-delete" => (REVIEW_ACTION_WOULD_DELETE, REVIEW_STATUS_READY, None),
        "would-fail-ambiguous-uid" | "would-fail-ambiguous-name" => (
            REVIEW_ACTION_BLOCKED_AMBIGUOUS,
            REVIEW_STATUS_BLOCKED,
            Some(REVIEW_REASON_AMBIGUOUS_LIVE_NAME_MATCH.to_string()),
        ),
        "would-fail-uid-name-mismatch" => (
            REVIEW_ACTION_BLOCKED_UID_MISMATCH,
            REVIEW_STATUS_BLOCKED,
            Some(REVIEW_REASON_UID_NAME_MISMATCH.to_string()),
        ),
        _ => (REVIEW_ACTION_BLOCKED, REVIEW_STATUS_BLOCKED, None),
    }
}

fn live_mutation_row_to_review_action(row: &[String]) -> ReviewMutationAction {
    let identity = live_mutation_review_identity(row);
    let action_id = live_mutation_review_action_id(row, &identity);
    let (action, status, blocked_reason) = normalize_live_mutation_review_action(row);
    ReviewMutationActionInput {
        action_id,
        action: action.to_string(),
        domain: "datasource".to_string(),
        resource_kind: "datasource".to_string(),
        identity,
        status: status.to_string(),
        blocked_reason,
        details: live_mutation_review_details(row),
        review_hints: Vec::new(),
        raw: build_live_mutation_row_raw(row),
    }
    .into()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_live_mutation_review_projection(
    rows: &[Vec<String>],
) -> DatasourceLiveMutationReviewProjection {
    let mut actions = rows
        .iter()
        .map(|row| live_mutation_row_to_review_action(row))
        .collect::<Vec<_>>();
    actions.sort_by(|left, right| {
        left.kind_order
            .cmp(&right.kind_order)
            .then_with(|| review_action_rank(&left.action).cmp(&review_action_rank(&right.action)))
            .then_with(|| left.identity.cmp(&right.identity))
            .then_with(|| left.action_id.cmp(&right.action_id))
    });
    DatasourceLiveMutationReviewProjection {
        domains: vec!["datasource"],
        actions,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_live_mutation_review_envelope(rows: &[Vec<String>]) -> ReviewMutationEnvelope {
    let projection = build_live_mutation_review_projection(rows);
    build_review_mutation_envelope(projection.actions, &projection.domains)
}

pub(crate) fn render_live_mutation_table(
    rows: &[Vec<String>],
    include_header: bool,
) -> Vec<String> {
    let headers = vec![
        "OPERATION".to_string(),
        "UID".to_string(),
        "NAME".to_string(),
        "TYPE".to_string(),
        "MATCH".to_string(),
        "ACTION".to_string(),
        "TARGET_ID".to_string(),
    ];
    let mut widths: Vec<usize> = headers.iter().map(|header| header.len()).collect();
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            widths[index] = widths[index].max(value.len());
        }
    }
    let format_row = |values: &[String]| -> String {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{:<width$}", value, width = widths[index]))
            .collect::<Vec<String>>()
            .join("  ")
    };
    let separator = widths
        .iter()
        .map(|width| "-".repeat(*width))
        .collect::<Vec<String>>();
    let mut lines = Vec::new();
    if include_header {
        lines.push(format_row(&headers));
        lines.push(format_row(&separator));
    }
    lines.extend(rows.iter().map(|row| format_row(row)));
    lines
}

pub(crate) fn render_live_mutation_json(rows: &[Vec<String>]) -> Value {
    let create_count = rows.iter().filter(|row| row[5] == "would-create").count();
    let update_count = rows.iter().filter(|row| row[5] == "would-update").count();
    let delete_count = rows.iter().filter(|row| row[5] == "would-delete").count();
    let blocked_count = rows
        .iter()
        .filter(|row| row[5].starts_with("would-fail-"))
        .count();
    Value::Object(Map::from_iter(vec![
        (
            "items".to_string(),
            Value::Array(
                rows.iter()
                    .map(|row| {
                        Value::Object(Map::from_iter(vec![
                            ("operation".to_string(), Value::String(row[0].clone())),
                            ("uid".to_string(), Value::String(row[1].clone())),
                            ("name".to_string(), Value::String(row[2].clone())),
                            ("type".to_string(), Value::String(row[3].clone())),
                            ("match".to_string(), Value::String(row[4].clone())),
                            ("action".to_string(), Value::String(row[5].clone())),
                            ("targetId".to_string(), Value::String(row[6].clone())),
                        ]))
                    })
                    .collect(),
            ),
        ),
        (
            "summary".to_string(),
            Value::Object(Map::from_iter(vec![
                (
                    "itemCount".to_string(),
                    Value::Number((rows.len() as i64).into()),
                ),
                (
                    "createCount".to_string(),
                    Value::Number((create_count as i64).into()),
                ),
                (
                    "updateCount".to_string(),
                    Value::Number((update_count as i64).into()),
                ),
                (
                    "deleteCount".to_string(),
                    Value::Number((delete_count as i64).into()),
                ),
                (
                    "blockedCount".to_string(),
                    Value::Number((blocked_count as i64).into()),
                ),
            ])),
        ),
    ]))
}

pub(crate) fn validate_live_mutation_dry_run_args(
    table: bool,
    json: bool,
    dry_run: bool,
    no_header: bool,
    verb: &str,
) -> Result<()> {
    if table && !dry_run {
        return Err(message(format!(
            "--table is only supported with --dry-run for datasource {verb}."
        )));
    }
    if json && !dry_run {
        return Err(message(format!(
            "--json is only supported with --dry-run for datasource {verb}."
        )));
    }
    if table && json {
        return Err(message(format!(
            "--table and --json are mutually exclusive for datasource {verb}."
        )));
    }
    if no_header && !table {
        return Err(message(format!(
            "--no-header is only supported with --dry-run --table for datasource {verb}."
        )));
    }
    Ok(())
}

pub(crate) fn render_import_table(
    rows: &[Vec<String>],
    include_header: bool,
    selected_columns: Option<&[String]>,
) -> Vec<String> {
    let columns = if let Some(selected) = selected_columns {
        if requested_columns_include_all(selected) {
            vec![
                (0usize, "UID"),
                (1usize, "NAME"),
                (2usize, "TYPE"),
                (3usize, "MATCH_BASIS"),
                (4usize, "DESTINATION"),
                (5usize, "ACTION"),
                (6usize, "ORG_ID"),
                (7usize, "FILE"),
                (8usize, "TARGET_UID"),
                (9usize, "TARGET_VERSION"),
                (10usize, "TARGET_READ_ONLY"),
                (11usize, "BLOCKED_REASON"),
            ]
        } else {
            selected
                .iter()
                .map(|column| match column.as_str() {
                    "uid" => (0usize, "UID"),
                    "name" => (1usize, "NAME"),
                    "type" => (2usize, "TYPE"),
                    "match_basis" => (3usize, "MATCH_BASIS"),
                    "destination" => (4usize, "DESTINATION"),
                    "action" => (5usize, "ACTION"),
                    "org_id" => (6usize, "ORG_ID"),
                    "file" => (7usize, "FILE"),
                    "target_uid" => (8usize, "TARGET_UID"),
                    "target_version" => (9usize, "TARGET_VERSION"),
                    "target_read_only" => (10usize, "TARGET_READ_ONLY"),
                    "blocked_reason" => (11usize, "BLOCKED_REASON"),
                    _ => unreachable!("validated datasource import output column"),
                })
                .collect::<Vec<(usize, &str)>>()
        }
    } else {
        vec![
            (0usize, "UID"),
            (1usize, "NAME"),
            (2usize, "TYPE"),
            (3usize, "MATCH_BASIS"),
            (4usize, "DESTINATION"),
            (5usize, "ACTION"),
            (6usize, "ORG_ID"),
            (7usize, "FILE"),
            (8usize, "TARGET_UID"),
            (9usize, "TARGET_VERSION"),
            (10usize, "TARGET_READ_ONLY"),
            (11usize, "BLOCKED_REASON"),
        ]
    };
    let headers = columns
        .iter()
        .map(|(_, header)| header.to_string())
        .collect::<Vec<String>>();
    let mut widths: Vec<usize> = headers.iter().map(|item| item.len()).collect();
    for row in rows {
        for (index, (source_index, _)) in columns.iter().enumerate() {
            let value = row.get(*source_index).map(String::as_str).unwrap_or("");
            widths[index] = widths[index].max(value.len());
        }
    }
    let format_row = |values: &[String]| -> String {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{:<width$}", value, width = widths[index]))
            .collect::<Vec<String>>()
            .join("  ")
    };
    let separator = widths
        .iter()
        .map(|width| "-".repeat(*width))
        .collect::<Vec<String>>();
    let mut lines = Vec::new();
    if include_header {
        lines.push(format_row(&headers));
        lines.push(format_row(&separator));
    }
    lines.extend(rows.iter().map(|row| {
        let values = columns
            .iter()
            .map(|(source_index, _)| row.get(*source_index).cloned().unwrap_or_default())
            .collect::<Vec<String>>();
        format_row(&values)
    }));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review_contract::{
        build_review_mutation_summary_rows, REVIEW_ACTION_BLOCKED_UID_MISMATCH,
        REVIEW_ACTION_WOULD_DELETE, REVIEW_REASON_UID_NAME_MISMATCH, REVIEW_STATUS_BLOCKED,
        REVIEW_STATUS_READY,
    };
    use serde_json::json;

    #[test]
    fn live_mutation_preview_review_projection_and_envelope_preserve_row_evidence() {
        let rows = vec![
            vec![
                "delete".to_string(),
                "prom-main".to_string(),
                "Prometheus Main".to_string(),
                "prometheus".to_string(),
                "exists-uid".to_string(),
                "would-delete".to_string(),
                "7".to_string(),
            ],
            vec![
                "add".to_string(),
                "loki-main".to_string(),
                "Loki Main".to_string(),
                "loki".to_string(),
                "uid-name-mismatch".to_string(),
                "would-fail-uid-name-mismatch".to_string(),
                "12".to_string(),
            ],
        ];

        let projection = build_live_mutation_review_projection(&rows);

        assert_eq!(projection.domains, vec!["datasource"]);
        assert_eq!(projection.actions.len(), 2);

        let blocked = &projection.actions[0];
        assert_eq!(
            blocked.action_id,
            "datasource-live-mutation:add:uid:loki-main:target:12"
        );
        assert_eq!(blocked.action, REVIEW_ACTION_BLOCKED_UID_MISMATCH);
        assert_eq!(blocked.status, REVIEW_STATUS_BLOCKED);
        assert_eq!(blocked.identity, "loki-main");
        assert_eq!(
            blocked.blocked_reason.as_deref(),
            Some(REVIEW_REASON_UID_NAME_MISMATCH)
        );
        assert_eq!(
            blocked.details.as_deref(),
            Some("operation=add match=uid-name-mismatch targetId=12")
        );
        assert_eq!(
            blocked.raw,
            json!({
                "operation": "add",
                "uid": "loki-main",
                "name": "Loki Main",
                "type": "loki",
                "match": "uid-name-mismatch",
                "action": "would-fail-uid-name-mismatch",
                "targetId": "12",
            })
        );

        let ready = &projection.actions[1];
        assert_eq!(
            ready.action_id,
            "datasource-live-mutation:delete:uid:prom-main:target:7"
        );
        assert_eq!(ready.action, REVIEW_ACTION_WOULD_DELETE);
        assert_eq!(ready.status, REVIEW_STATUS_READY);
        assert_eq!(ready.identity, "prom-main");
        assert_eq!(ready.blocked_reason, None);
        assert_eq!(
            ready.details.as_deref(),
            Some("operation=delete match=exists-uid targetId=7")
        );
        assert_eq!(
            ready.raw,
            json!({
                "operation": "delete",
                "uid": "prom-main",
                "name": "Prometheus Main",
                "type": "prometheus",
                "match": "exists-uid",
                "action": "would-delete",
                "targetId": "7",
            })
        );

        let envelope = build_live_mutation_review_envelope(&rows);
        assert_eq!(envelope.actions, projection.actions);
        assert_eq!(envelope.summary.action_count, 2);
        assert_eq!(envelope.summary.blocked_count, 1);
        assert_eq!(envelope.domains.len(), 1);
        assert_eq!(envelope.domains[0].id, "datasource");
        assert_eq!(envelope.domains[0].delete, 1);
        assert_eq!(envelope.domains[0].blocked, 1);
        assert_eq!(
            envelope.blocked_reasons,
            vec![REVIEW_REASON_UID_NAME_MISMATCH.to_string()]
        );
    }

    #[test]
    fn datasource_live_mutation_review_envelope_feeds_shared_summary_rows_without_json_drift() {
        let rows = vec![vec![
            "modify".to_string(),
            "prom-main".to_string(),
            "Prometheus Main".to_string(),
            "prometheus".to_string(),
            "exists-uid".to_string(),
            "would-update".to_string(),
            "7".to_string(),
        ]];
        let public_json_before = render_live_mutation_json(&rows);

        let envelope = build_live_mutation_review_envelope(&rows);
        let summary_rows = build_review_mutation_summary_rows(&envelope);

        assert_eq!(summary_rows.len(), 1);
        assert_eq!(summary_rows[0].domain, "datasource");
        assert_eq!(summary_rows[0].resource_kind, "datasource");
        assert_eq!(summary_rows[0].identity, "prom-main");
        assert_eq!(summary_rows[0].action, "would-update");
        assert_eq!(summary_rows[0].status, REVIEW_STATUS_READY);
        assert_eq!(
            summary_rows[0].details.as_deref(),
            Some("operation=modify match=exists-uid targetId=7")
        );
        assert_eq!(summary_rows[0].action_count, 1);
        assert_eq!(summary_rows[0].domain_count, 1);
        assert_eq!(summary_rows[0].blocked_count, 0);
        assert_eq!(summary_rows[0].warning_count, 0);
        assert!(summary_rows[0].blocked_reasons.is_empty());
        assert_eq!(render_live_mutation_json(&rows), public_json_before);
        assert_eq!(
            public_json_before["items"][0],
            json!({
                "operation": "modify",
                "uid": "prom-main",
                "name": "Prometheus Main",
                "type": "prometheus",
                "match": "exists-uid",
                "action": "would-update",
                "targetId": "7",
            })
        );
    }
}
