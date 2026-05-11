use super::{
    build_alert_plan_with_request, write_new_rule_scaffold, write_new_template_scaffold,
    CONTACT_POINT_KIND, MUTE_TIMING_KIND, POLICIES_KIND, RULE_KIND, TEMPLATE_KIND,
    TOOL_SCHEMA_VERSION,
};
use crate::review_contract::build_review_mutation_summary_rows;
use reqwest::Method;
use serde_json::json;
use tempfile::tempdir;

#[test]
fn build_alert_plan_with_request_generates_create_update_noop_and_blocked_rows() {
    let temp = tempdir().unwrap();
    write_new_rule_scaffold(
        &temp.path().join("rules/create-rule.json"),
        "create-rule",
        true,
    )
    .unwrap();
    super::write_pretty_json(
        &temp.path().join("contact-points/update-contact-point.yaml"),
        &json!({
            "kind": CONTACT_POINT_KIND,
            "apiVersion": super::TOOL_API_VERSION,
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "spec": {
                "uid": "cp-update",
                "name": "Update Me",
                "type": "webhook",
                "settings": {"url": "http://127.0.0.1/new"}
            }
        }),
    );
    write_new_template_scaffold(
        &temp.path().join("templates/example-template.json"),
        "example-template",
        true,
    )
    .unwrap();

    let plan = build_alert_plan_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/v1/provisioning/alert-rules/create-rule") => Ok(None),
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([
                {
                    "uid": "cp-update",
                    "name": "Update Me",
                    "type": "webhook",
                    "settings": {"url": "http://127.0.0.1/old"}
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/templates/example-template") => Ok(Some(json!({
                "name": "example-template",
                "template": "{{ define \"example-template\" }}replace me{{ end }}"
            }))),
            (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([
                {
                    "name": "off-hours",
                    "time_intervals": []
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(json!([
                {
                    "name": "example-template",
                    "template": "{{ define \"example-template\" }}replace me{{ end }}"
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/policies") => Ok(Some(json!({
                "receiver": "grafana-default-email"
            }))),
            _ => panic!("unexpected request {method:?} {path}"),
        },
        temp.path(),
        false,
    )
    .unwrap();

    assert_eq!(plan["summary"]["create"], json!(1));
    assert_eq!(plan["summary"]["update"], json!(1));
    assert_eq!(plan["summary"]["noop"], json!(1));
    assert_eq!(plan["summary"]["blocked"], json!(2));
    assert_eq!(plan["summary"]["warning"], json!(0));

    let rows = plan["rows"].as_array().unwrap();
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(RULE_KIND)
            && row["identity"] == json!("create-rule")
            && row["action"] == json!("create")
            && row["status"] == json!("ready")
            && row["actionId"].as_str().unwrap_or("").ends_with("::create")
            && row["changedFields"].is_array()
            && row["changes"].is_array()
    }));
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(CONTACT_POINT_KIND)
            && row["identity"] == json!("cp-update")
            && row["action"] == json!("update")
            && row["status"] == json!("ready")
            && row["changedFields"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == "settings")
            && row["changes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|change| change["field"] == json!("settings"))
    }));
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(TEMPLATE_KIND)
            && row["identity"] == json!("example-template")
            && row["action"] == json!("noop")
            && row["status"] == json!("same")
            && row["reviewHints"] == json!([])
    }));
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(MUTE_TIMING_KIND)
            && row["identity"] == json!("off-hours")
            && row["action"] == json!("blocked")
            && row["status"] == json!("blocked")
            && row["reason"] == json!("prune-required")
            && row["blockedReason"] == json!("prune-required")
    }));
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(POLICIES_KIND)
            && row["identity"] == json!("grafana-default-email")
            && row["action"] == json!("blocked")
            && row["status"] == json!("blocked")
    }));
}

#[test]
fn build_alert_plan_with_request_surfaces_linked_rule_review_hints() {
    let temp = tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join("rules")).unwrap();
    std::fs::write(
        temp.path().join("rules/linked-rule.json"),
        serde_json::to_string_pretty(&json!({
            "uid": "linked-rule",
            "title": "Linked Rule",
            "folderUID": "alerts",
            "ruleGroup": "linked",
            "condition": "A",
            "data": [],
            "annotations": {
                "__dashboardUid__": "src-dash",
                "__panelId__": "7"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let plan = build_alert_plan_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/v1/provisioning/alert-rules/linked-rule") => Ok(Some(json!({
                "uid": "linked-rule",
                "title": "Linked Rule",
                "folderUID": "alerts",
                "ruleGroup": "linked",
                "condition": "A",
                "data": [],
                "annotations": {
                    "__dashboardUid__": "src-dash",
                    "__panelId__": "7"
                }
            }))),
            (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/policies") => Ok(Some(json!({"receiver": "root"}))),
            _ => panic!("unexpected request {method:?} {path}"),
        },
        temp.path(),
        false,
    )
    .unwrap();

    assert_eq!(plan["summary"]["warning"], json!(1));
    let row = plan["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["identity"] == json!("linked-rule"))
        .unwrap();
    assert_eq!(row["action"], json!("noop"));
    assert_eq!(row["status"], json!("warning"));
    assert!(row["actionId"]
        .as_str()
        .unwrap_or("")
        .contains("linked-rule"));
    let hints = row["reviewHints"].as_array().unwrap();
    assert!(hints
        .iter()
        .any(|hint| hint["code"] == json!("linked-dashboard-reference")));
    assert!(hints
        .iter()
        .any(|hint| hint["code"] == json!("linked-panel-reference")));
    assert!(row["changedFields"].as_array().unwrap().is_empty());
}

#[test]
fn build_alert_plan_with_request_marks_live_only_resources_delete_when_prune_enabled() {
    let temp = tempdir().unwrap();
    write_new_rule_scaffold(
        &temp.path().join("rules/create-rule.json"),
        "create-rule",
        true,
    )
    .unwrap();

    let plan = build_alert_plan_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/v1/provisioning/alert-rules/create-rule") => Ok(None),
            (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([
                {
                    "uid": "cp-delete",
                    "name": "Delete Me",
                    "type": "webhook",
                    "settings": {"url": "http://127.0.0.1/delete"}
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/policies") => Ok(None),
            _ => panic!("unexpected request {method:?} {path}"),
        },
        temp.path(),
        true,
    )
    .unwrap();

    assert!(plan["rows"].as_array().unwrap().iter().any(|row| {
        row["kind"] == json!(CONTACT_POINT_KIND)
            && row["identity"] == json!("cp-delete")
            && row["action"] == json!("delete")
    }));
}

#[test]
fn build_alert_plan_with_request_treats_authoring_round_trip_defaults_as_noop() {
    let temp = tempdir().unwrap();
    super::write_pretty_json(
        &temp
            .path()
            .join("contact-points/authoring-contact-point.json"),
        &json!({
            "kind": CONTACT_POINT_KIND,
            "apiVersion": super::TOOL_API_VERSION,
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "spec": {
                "uid": "cp-authoring",
                "name": "Authoring Webhook",
                "type": "webhook",
                "settings": {"url": "http://127.0.0.1/notify"}
            }
        }),
    );
    super::write_pretty_json(
        &temp.path().join("policies/notification-policies.json"),
        &json!({
            "kind": POLICIES_KIND,
            "apiVersion": super::TOOL_API_VERSION,
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "spec": {
                "receiver": "pagerduty-primary",
                "group_by": ["grafana_folder", "alertname"],
                "routes": [{
                    "receiver": "pagerduty-primary",
                    "continue": false,
                    "group_by": ["grafana_folder", "alertname"],
                    "object_matchers": [
                        ["team", "=", "platform"],
                        ["severity", "=", "critical"],
                        ["grafana_utils_route", "=", "pagerduty-primary"]
                    ]
                }]
            }
        }),
    );
    super::write_pretty_json(
        &temp.path().join("rules/cpu-high.json"),
        &json!({
            "kind": RULE_KIND,
            "apiVersion": super::TOOL_API_VERSION,
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "spec": {
                "uid": "cpu-high",
                "title": "cpu-high",
                "folderUID": "platform-alerts",
                "ruleGroup": "cpu",
                "condition": "A",
                "for": "5m",
                "noDataState": "NoData",
                "execErrState": "Alerting",
                "labels": {
                    "grafana_utils_route": "pagerduty-primary",
                    "severity": "critical",
                    "team": "platform"
                },
                "annotations": {},
                "data": [{
                    "refId": "A",
                    "datasourceUid": "__expr__",
                    "relativeTimeRange": {"from": 0, "to": 0},
                    "model": {
                        "refId": "A",
                        "type": "classic_conditions",
                        "datasource": {"type": "__expr__", "uid": "__expr__"},
                        "expression": "A",
                        "conditions": [{
                            "type": "query",
                            "query": {"params": ["A"]},
                            "reducer": {"type": "last", "params": []},
                            "evaluator": {"type": "gt", "params": [80.0]},
                            "operator": {"type": "and"}
                        }],
                        "intervalMs": 1000,
                        "maxDataPoints": 43200
                    }
                }]
            }
        }),
    );

    let plan = build_alert_plan_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([
                {
                    "uid": "cp-authoring",
                    "name": "Authoring Webhook",
                    "type": "webhook",
                    "settings": {"url": "http://127.0.0.1/notify"},
                    "disableResolveMessage": false
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/policies") => Ok(Some(json!({
                "receiver": "pagerduty-primary",
                "group_by": ["grafana_folder", "alertname"],
                "routes": [{
                    "receiver": "pagerduty-primary",
                    "group_by": ["grafana_folder", "alertname"],
                    "object_matchers": [
                        ["grafana_utils_route", "=", "pagerduty-primary"],
                        ["severity", "=", "critical"],
                        ["team", "=", "platform"]
                    ]
                }]
            }))),
            (Method::GET, "/api/v1/provisioning/alert-rules/cpu-high") => Ok(Some(json!({
                "uid": "cpu-high",
                "title": "cpu-high",
                "folderUID": "platform-alerts",
                "ruleGroup": "cpu",
                "condition": "A",
                "for": "5m",
                "noDataState": "NoData",
                "execErrState": "Alerting",
                "isPaused": false,
                "keep_firing_for": "0s",
                "notification_settings": null,
                "record": null,
                "orgID": 1,
                "data": [{
                    "refId": "A",
                    "queryType": "",
                    "datasourceUid": "__expr__",
                    "relativeTimeRange": {"from": 0, "to": 0},
                    "model": {
                        "refId": "A",
                        "type": "classic_conditions",
                        "datasource": {"type": "__expr__", "uid": "__expr__"},
                        "expression": "A",
                        "conditions": [{
                            "type": "query",
                            "query": {"params": ["A"]},
                            "reducer": {"type": "last", "params": []},
                            "evaluator": {"type": "gt", "params": [80.0]},
                            "operator": {"type": "and"}
                        }],
                        "intervalMs": 1000,
                        "maxDataPoints": 43200
                    }
                }],
                "labels": {
                    "grafana_utils_route": "pagerduty-primary",
                    "severity": "critical",
                    "team": "platform"
                },
                "annotations": {}
            }))),
            (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([
                {"uid": "cpu-high"}
            ]))),
            (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(json!([]))),
            _ => panic!("unexpected request {method:?} {path}"),
        },
        temp.path(),
        true,
    )
    .unwrap();

    assert_eq!(plan["summary"]["create"], json!(0));
    assert_eq!(plan["summary"]["update"], json!(0));
    assert_eq!(plan["summary"]["noop"], json!(3));
    assert_eq!(plan["summary"]["delete"], json!(0));
}

#[test]
fn alert_plan_review_projection_maps_local_actions_without_changing_raw_rows() {
    let create_row = json!({
        "domain": "alert",
        "resourceKind": RULE_KIND,
        "kind": RULE_KIND,
        "identity": "create-rule",
        "actionId": "grafana-alert-rule::create-rule::create",
        "action": "create",
        "status": "ready",
        "reason": "missing-live",
        "blockedReason": null,
        "reviewHints": [],
        "changedFields": ["uid", "title"],
        "changes": [],
        "path": "/tmp/rules/create-rule.json",
        "desired": {"uid": "create-rule", "title": "Create Rule"},
        "live": null
    });
    let update_row = json!({
        "domain": "alert",
        "resourceKind": CONTACT_POINT_KIND,
        "kind": CONTACT_POINT_KIND,
        "identity": "cp-update",
        "actionId": "grafana-contact-point::cp-update::update",
        "action": "update",
        "status": "ready",
        "reason": "drift-detected",
        "blockedReason": null,
        "reviewHints": [],
        "changedFields": ["settings"],
        "changes": [],
        "path": "/tmp/contact-points/cp-update.json",
        "desired": {"uid": "cp-update"},
        "live": {"uid": "cp-update"}
    });
    let noop_row = json!({
        "domain": "alert",
        "resourceKind": RULE_KIND,
        "kind": RULE_KIND,
        "identity": "linked-rule",
        "actionId": "grafana-alert-rule::linked-rule::noop",
        "action": "noop",
        "status": "warning",
        "reason": "in-sync",
        "blockedReason": null,
        "reviewHints": [
            {
                "code": "linked-dashboard-reference",
                "field": "annotations.__dashboardUid__",
                "before": "src-dash",
                "after": "src-dash"
            },
            {
                "code": "linked-panel-reference",
                "field": "annotations.__panelId__",
                "before": "7",
                "after": "7"
            }
        ],
        "changedFields": [],
        "changes": [],
        "path": "/tmp/rules/linked-rule.json",
        "desired": {"uid": "linked-rule"},
        "live": {"uid": "linked-rule"}
    });
    let delete_row = json!({
        "domain": "alert",
        "resourceKind": TEMPLATE_KIND,
        "kind": TEMPLATE_KIND,
        "identity": "remote-template",
        "actionId": "grafana-notification-template::remote-template::delete",
        "action": "delete",
        "status": "ready",
        "reason": "missing-from-desired-state",
        "blockedReason": null,
        "reviewHints": [],
        "changedFields": ["name"],
        "changes": [],
        "path": null,
        "desired": null,
        "live": {"name": "remote-template"}
    });
    let blocked_row = json!({
        "domain": "alert",
        "resourceKind": MUTE_TIMING_KIND,
        "kind": MUTE_TIMING_KIND,
        "identity": "off-hours",
        "actionId": "grafana-mute-timing::off-hours::blocked",
        "action": "blocked",
        "status": "blocked",
        "reason": "prune-required",
        "blockedReason": "prune-required",
        "reviewHints": [],
        "changedFields": ["name"],
        "changes": [],
        "path": null,
        "desired": null,
        "live": {"name": "off-hours"}
    });
    let plan = super::build_alert_plan_document(
        &[
            create_row.clone(),
            update_row.clone(),
            noop_row.clone(),
            delete_row.clone(),
            blocked_row.clone(),
        ],
        true,
    );

    let projection =
        super::alert_runtime_support::build_alert_plan_review_projection(&plan).unwrap();

    assert_eq!(projection.domains, vec!["alert"]);
    assert_eq!(projection.actions.len(), 5);

    let create = projection
        .actions
        .iter()
        .find(|action| action.identity == "create-rule")
        .unwrap();
    assert_eq!(create.action, "would-create");
    assert_eq!(create.domain, "alert");
    assert_eq!(create.resource_kind, RULE_KIND);
    assert_eq!(create.identity, "create-rule");
    assert_eq!(create.status, "ready");
    assert_eq!(create.blocked_reason, None);
    assert_eq!(create.review_hints, Vec::<String>::new());
    assert_eq!(create.raw, create_row);

    let update = projection
        .actions
        .iter()
        .find(|action| action.identity == "cp-update")
        .unwrap();
    assert_eq!(update.action, "would-update");
    assert_eq!(update.domain, "alert");
    assert_eq!(update.resource_kind, CONTACT_POINT_KIND);
    assert_eq!(update.identity, "cp-update");
    assert_eq!(update.status, "ready");
    assert_eq!(update.blocked_reason, None);
    assert_eq!(update.review_hints, Vec::<String>::new());
    assert_eq!(update.raw, update_row);

    let noop = projection
        .actions
        .iter()
        .find(|action| action.identity == "linked-rule")
        .unwrap();
    assert_eq!(noop.action, "same");
    assert_eq!(noop.domain, "alert");
    assert_eq!(noop.resource_kind, RULE_KIND);
    assert_eq!(noop.identity, "linked-rule");
    assert_eq!(noop.status, "warning");
    assert_eq!(noop.blocked_reason, None);
    assert_eq!(
        noop.review_hints,
        vec![
            "linked-dashboard-reference".to_string(),
            "linked-panel-reference".to_string()
        ]
    );
    assert_eq!(noop.raw, noop_row);

    let delete = projection
        .actions
        .iter()
        .find(|action| action.identity == "remote-template")
        .unwrap();
    assert_eq!(delete.action, "would-delete");
    assert_eq!(delete.domain, "alert");
    assert_eq!(delete.resource_kind, TEMPLATE_KIND);
    assert_eq!(delete.identity, "remote-template");
    assert_eq!(delete.status, "ready");
    assert_eq!(delete.blocked_reason, None);
    assert_eq!(delete.review_hints, Vec::<String>::new());
    assert_eq!(delete.raw, delete_row);

    let blocked = projection
        .actions
        .iter()
        .find(|action| action.identity == "off-hours")
        .unwrap();
    assert_eq!(blocked.action, "blocked");
    assert_eq!(blocked.domain, "alert");
    assert_eq!(blocked.resource_kind, MUTE_TIMING_KIND);
    assert_eq!(blocked.identity, "off-hours");
    assert_eq!(blocked.status, "blocked");
    assert_eq!(blocked.blocked_reason.as_deref(), Some("prune-required"));
    assert_eq!(blocked.review_hints, Vec::<String>::new());
    assert_eq!(blocked.raw, blocked_row);

    let envelope = super::alert_runtime_support::build_alert_plan_review_envelope(&plan).unwrap();

    assert_eq!(envelope.summary.action_count, 5);
    assert_eq!(envelope.summary.domain_count, 1);
    assert_eq!(envelope.summary.same_count, 1);
    assert_eq!(envelope.summary.warning_count, 1);
    assert_eq!(envelope.summary.blocked_count, 1);
    assert_eq!(envelope.blocked_reasons, vec!["prune-required"]);
    assert_eq!(envelope.domains.len(), 1);
    assert_eq!(envelope.domains[0].id, "alert");
    assert_eq!(envelope.domains[0].checked, 5);
    assert_eq!(envelope.domains[0].same, 1);
    assert_eq!(envelope.domains[0].create, 1);
    assert_eq!(envelope.domains[0].update, 1);
    assert_eq!(envelope.domains[0].delete, 1);
    assert_eq!(envelope.domains[0].warning, 1);
    assert_eq!(envelope.domains[0].blocked, 1);
}

#[test]
fn alert_plan_review_envelope_feeds_shared_summary_rows_without_public_json_drift() {
    let update_row = json!({
        "domain": "alert",
        "resourceKind": CONTACT_POINT_KIND,
        "kind": CONTACT_POINT_KIND,
        "identity": "cp-update",
        "actionId": "grafana-contact-point::cp-update::update",
        "action": "update",
        "status": "ready",
        "reason": "drift-detected",
        "blockedReason": null,
        "reviewHints": [],
        "changedFields": ["settings"],
        "changes": [],
        "path": "/tmp/contact-points/cp-update.json",
        "desired": {"uid": "cp-update"},
        "live": {"uid": "cp-update"}
    });
    let plan = super::build_alert_plan_document(std::slice::from_ref(&update_row), false);
    let public_rows_before = plan["rows"].clone();

    let envelope = super::alert_runtime_support::build_alert_plan_review_envelope(&plan).unwrap();
    let summary_rows = build_review_mutation_summary_rows(&envelope);

    assert_eq!(summary_rows.len(), 1);
    assert_eq!(summary_rows[0].domain, "alert");
    assert_eq!(summary_rows[0].resource_kind, CONTACT_POINT_KIND);
    assert_eq!(summary_rows[0].identity, "cp-update");
    assert_eq!(summary_rows[0].action, "would-update");
    assert_eq!(summary_rows[0].status, "ready");
    assert_eq!(summary_rows[0].details.as_deref(), Some("fields=settings"));
    assert_eq!(summary_rows[0].action_count, 1);
    assert_eq!(summary_rows[0].domain_count, 1);
    assert_eq!(summary_rows[0].blocked_count, 0);
    assert_eq!(summary_rows[0].warning_count, 0);
    assert!(summary_rows[0].blocked_reasons.is_empty());
    assert_eq!(plan["rows"], public_rows_before);
    assert_eq!(plan["rows"][0], update_row);
}
