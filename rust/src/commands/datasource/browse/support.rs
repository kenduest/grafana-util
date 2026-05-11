#![cfg_attr(not(any(feature = "tui", test)), allow(dead_code))]

use serde_json::{Map, Value};

use crate::common::{message, string_field, Result};
use crate::dashboard::{build_auth_context, build_http_client_for_org, DEFAULT_ORG_ID};
use crate::datasource_secret::{collect_secret_placeholders, iter_secret_placeholder_names};
use crate::http::JsonHttpClient;

use super::DatasourceBrowseArgs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DatasourceBrowseItemKind {
    Org,
    Datasource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DatasourceBrowseItem {
    pub(crate) kind: DatasourceBrowseItemKind,
    pub(crate) depth: u16,
    pub(crate) id: i64,
    pub(crate) uid: String,
    pub(crate) name: String,
    pub(crate) datasource_type: String,
    pub(crate) access: String,
    pub(crate) url: String,
    pub(crate) is_default: bool,
    pub(crate) org: String,
    pub(crate) org_id: String,
    pub(crate) details: Map<String, Value>,
    pub(crate) datasource_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DatasourceBrowseDocument {
    pub(crate) scope_label: String,
    pub(crate) org: String,
    pub(crate) org_id: String,
    pub(crate) items: Vec<DatasourceBrowseItem>,
    pub(crate) org_count: usize,
    pub(crate) datasource_count: usize,
}

impl DatasourceBrowseItem {
    pub(crate) fn is_org_row(&self) -> bool {
        self.kind == DatasourceBrowseItemKind::Org
    }
}

pub(crate) fn load_datasource_browse_document(
    client: &JsonHttpClient,
    args: &DatasourceBrowseArgs,
) -> Result<DatasourceBrowseDocument> {
    if args.all_orgs {
        return load_all_orgs_document(&args.common, client);
    }
    load_single_org_document(client)
}

pub(crate) fn detail_lines(item: &DatasourceBrowseItem) -> Vec<String> {
    if item.is_org_row() {
        return vec![
            format!("Org: {}", display_value(&item.org, "-")),
            format!("Org ID: {}", display_value(&item.org_id, "-")),
            format!("Datasources: {}", item.datasource_count),
        ];
    }

    let mut lines = vec![
        format!("ID: {}", item.id),
        format!("UID: {}", display_value(&item.uid, "-")),
        format!("Name: {}", display_value(&item.name, "-")),
        format!("Type: {}", display_value(&item.datasource_type, "-")),
        format!("URL: {}", display_value(&item.url, "-")),
        format!("Access: {}", display_value(&item.access, "-")),
        format!(
            "Default: {}",
            if item.is_default { "true" } else { "false" }
        ),
        format!("Org: {}", display_value(&item.org, "-")),
        format!("Org ID: {}", display_value(&item.org_id, "-")),
    ];

    if let Some(user) = item.details.get("user").and_then(Value::as_str) {
        if !user.trim().is_empty() {
            lines.push(format!("User: {}", user.trim()));
        }
    }
    if let Some(value) = item.details.get("basicAuth").and_then(Value::as_bool) {
        lines.push(format!("Basic auth: {value}"));
    }
    if let Some(value) = item.details.get("withCredentials").and_then(Value::as_bool) {
        lines.push(format!("With credentials: {value}"));
    }
    if let Some(database) = item.details.get("database").and_then(Value::as_str) {
        if !database.trim().is_empty() {
            lines.push(format!("Database: {}", database.trim()));
        }
    }
    if let Some(json_data) = item.details.get("jsonData").and_then(Value::as_object) {
        if !json_data.is_empty() {
            let keys = sorted_object_keys(json_data).join(", ");
            lines.push(format!("jsonData keys: {keys}"));
        }
    }
    if let Some(secure_json_fields) = item
        .details
        .get("secureJsonFields")
        .and_then(Value::as_object)
    {
        if !secure_json_fields.is_empty() {
            let keys = sorted_object_keys(secure_json_fields).join(", ");
            lines.push(format!("secureJsonFields: {keys}"));
        }
    }
    lines.extend(secret_review_lines(&item.details));

    lines
}

fn secret_review_lines(details: &Map<String, Value>) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(placeholders) = details
        .get("secureJsonDataPlaceholders")
        .and_then(Value::as_object)
    {
        lines.extend(placeholder_review_lines(placeholders));
    }
    if let Some(secure_json_fields) = details.get("secureJsonFields").and_then(Value::as_object) {
        lines.extend(live_secure_json_fields_review_lines(secure_json_fields));
    }
    if let Some(secure_json_data) = details.get("secureJsonData").and_then(Value::as_object) {
        lines.extend(resolved_secure_json_data_review_lines(secure_json_data));
    }
    lines
}

fn placeholder_review_lines(placeholders: &Map<String, Value>) -> Vec<String> {
    match collect_secret_placeholders(Some(placeholders)) {
        Ok(placeholders) if placeholders.is_empty() => Vec::new(),
        Ok(placeholders) => {
            let field_names = placeholders
                .iter()
                .map(|placeholder| placeholder.field_name.clone())
                .collect::<Vec<_>>();
            let placeholder_names = iter_secret_placeholder_names(&placeholders)
                .map(str::to_string)
                .collect::<Vec<_>>();
            vec![
                format!(
                    "Secret placeholders: available ({} field(s): {})",
                    field_names.len(),
                    field_names.join(", ")
                ),
                format!("Secret placeholder names: {}", placeholder_names.join(", ")),
                "Secret blocker status: blocked until --secret-values resolves placeholders"
                    .to_string(),
                "Secret review required: true (placeholder-backed secureJsonData)".to_string(),
            ]
        }
        Err(error) => vec![
            format!(
                "Secret placeholders: invalid placeholder contract ({} field(s): {})",
                placeholders.len(),
                sorted_object_keys(placeholders).join(", ")
            ),
            format!("Secret blocker status: blocked by placeholder contract error: {error}"),
            "Secret review required: true (placeholder contract error)".to_string(),
        ],
    }
}

fn live_secure_json_fields_review_lines(secure_json_fields: &Map<String, Value>) -> Vec<String> {
    let field_names = secure_json_fields
        .iter()
        .filter(|&(_, value)| value.as_bool().unwrap_or(false))
        .map(|(field_name, _)| field_name.to_string())
        .collect::<Vec<_>>();
    if field_names.is_empty() {
        return Vec::new();
    }
    vec![
        format!(
            "Secret placeholders: unavailable from live secureJsonFields ({} field(s): {})",
            field_names.len(),
            field_names.join(", ")
        ),
        "Secret blocker status: review-required; resolved values are hidden by Grafana".to_string(),
        "Secret review required: true (secure fields present)".to_string(),
    ]
}

fn resolved_secure_json_data_review_lines(secure_json_data: &Map<String, Value>) -> Vec<String> {
    let field_names = sorted_object_keys(secure_json_data);
    if field_names.is_empty() {
        return Vec::new();
    }
    vec![
        format!(
            "Secret material: hidden ({} secureJsonData field(s): {})",
            field_names.len(),
            field_names.join(", ")
        ),
        "Secret blocker status: review-required; resolved credential values are never displayed"
            .to_string(),
        "Secret review required: true (resolved secureJsonData hidden)".to_string(),
    ]
}

fn sorted_object_keys(object: &Map<String, Value>) -> Vec<String> {
    let mut keys = object.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}

pub(crate) fn build_modify_updates_from_browse(
    item: &DatasourceBrowseItem,
    name: &str,
    url: &str,
    access: &str,
    is_default: bool,
) -> Map<String, Value> {
    let mut updates = Map::new();
    if name.trim() != item.name {
        updates.insert("name".to_string(), Value::String(name.trim().to_string()));
    }
    if url.trim() != item.url {
        updates.insert("url".to_string(), Value::String(url.trim().to_string()));
    }
    if access.trim() != item.access {
        updates.insert(
            "access".to_string(),
            Value::String(access.trim().to_string()),
        );
    }
    if is_default != item.is_default {
        updates.insert("isDefault".to_string(), Value::Bool(is_default));
    }
    updates
}

pub(crate) fn fetch_datasource_by_uid(
    client: &JsonHttpClient,
    uid: &str,
) -> Result<Map<String, Value>> {
    super::fetch_datasource_by_uid_if_exists(client, uid)?.ok_or_else(|| {
        message(format!(
            "Datasource browse could not find live datasource UID {uid}."
        ))
    })
}

fn load_single_org_document(client: &JsonHttpClient) -> Result<DatasourceBrowseDocument> {
    let org = super::fetch_current_org(client)?;
    let org_name = string_field(&org, "name", "");
    let org_id = org
        .get("id")
        .map(|value| value.to_string())
        .unwrap_or_else(|| DEFAULT_ORG_ID.to_string());
    let items = datasource_rows_for_org(client, &org_name, &org_id, 0)?;
    let datasource_count = items.len();
    Ok(DatasourceBrowseDocument {
        scope_label: format!(
            "Org {} (id={})",
            display_value(&org_name, "-"),
            display_value(&org_id, "-")
        ),
        org: org_name,
        org_id,
        items,
        org_count: 1,
        datasource_count,
    })
}

fn load_all_orgs_document(
    common: &super::CommonCliArgs,
    client: &JsonHttpClient,
) -> Result<DatasourceBrowseDocument> {
    let context = build_auth_context(common)?;
    if context.auth_mode != "basic" {
        return Err(message(
            "Datasource browse with --all-orgs requires Basic auth (--basic-user / --basic-password).",
        ));
    }

    let mut orgs = super::list_orgs(client)?;
    orgs.sort_by(|left, right| {
        string_field(left, "name", "")
            .to_ascii_lowercase()
            .cmp(&string_field(right, "name", "").to_ascii_lowercase())
            .then_with(|| {
                left.get("id")
                    .map(Value::to_string)
                    .cmp(&right.get("id").map(Value::to_string))
            })
    });

    let mut items = Vec::new();
    let mut datasource_count = 0usize;
    for org in &orgs {
        let org_name = string_field(org, "name", "");
        let org_id = org.get("id").and_then(Value::as_i64).unwrap_or(1);
        let org_id_text = org_id.to_string();
        let scoped_client = build_http_client_for_org(common, org_id)?;
        let datasource_items = datasource_rows_for_org(&scoped_client, &org_name, &org_id_text, 1)?;
        datasource_count += datasource_items.len();
        items.push(org_row(
            org_name,
            org_id_text,
            datasource_items.len(),
            org.clone(),
        ));
        items.extend(datasource_items);
    }

    Ok(DatasourceBrowseDocument {
        scope_label: "All visible orgs".to_string(),
        org: "All visible orgs".to_string(),
        org_id: "-".to_string(),
        items,
        org_count: orgs.len(),
        datasource_count,
    })
}

fn datasource_rows_for_org(
    client: &JsonHttpClient,
    org_name: &str,
    org_id: &str,
    depth: u16,
) -> Result<Vec<DatasourceBrowseItem>> {
    let mut items = super::build_list_records(client)?
        .into_iter()
        .map(|record| datasource_row(record, org_name, org_id, depth))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .is_default
            .cmp(&left.is_default)
            .then_with(|| {
                left.name
                    .to_ascii_lowercase()
                    .cmp(&right.name.to_ascii_lowercase())
            })
            .then_with(|| {
                left.uid
                    .to_ascii_lowercase()
                    .cmp(&right.uid.to_ascii_lowercase())
            })
    });
    Ok(items)
}

fn datasource_row(
    record: Map<String, Value>,
    org_name: &str,
    org_id: &str,
    depth: u16,
) -> DatasourceBrowseItem {
    DatasourceBrowseItem {
        kind: DatasourceBrowseItemKind::Datasource,
        depth,
        id: record.get("id").and_then(Value::as_i64).unwrap_or_default(),
        uid: string_field(&record, "uid", ""),
        name: string_field(&record, "name", ""),
        datasource_type: string_field(&record, "type", ""),
        access: string_field(&record, "access", ""),
        url: string_field(&record, "url", ""),
        is_default: record
            .get("isDefault")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        org: string_field(&record, "org", org_name),
        org_id: string_field(&record, "orgId", org_id),
        details: record,
        datasource_count: 0,
    }
}

fn org_row(
    org_name: String,
    org_id: String,
    datasource_count: usize,
    details: Map<String, Value>,
) -> DatasourceBrowseItem {
    DatasourceBrowseItem {
        kind: DatasourceBrowseItemKind::Org,
        depth: 0,
        id: 0,
        uid: String::new(),
        name: org_name.clone(),
        datasource_type: "org".to_string(),
        access: String::new(),
        url: String::new(),
        is_default: false,
        org: org_name,
        org_id,
        details,
        datasource_count,
    }
}

fn display_value<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn datasource_item(details: Map<String, Value>) -> DatasourceBrowseItem {
        DatasourceBrowseItem {
            kind: DatasourceBrowseItemKind::Datasource,
            depth: 0,
            id: 12,
            uid: "prom-main".to_string(),
            name: "Prometheus Main".to_string(),
            datasource_type: "prometheus".to_string(),
            access: "proxy".to_string(),
            url: "http://prometheus".to_string(),
            is_default: true,
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            details,
            datasource_count: 0,
        }
    }

    #[test]
    fn detail_lines_render_live_secure_json_fields_as_review_required_placeholders() {
        let item = datasource_item(
            json!({
                "secureJsonFields": {
                    "httpHeaderValue1": true,
                    "basicAuthPassword": true
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = detail_lines(&item);

        assert!(lines.contains(
            &"Secret placeholders: unavailable from live secureJsonFields (2 field(s): basicAuthPassword, httpHeaderValue1)".to_string()
        ));
        assert!(lines.contains(
            &"Secret blocker status: review-required; resolved values are hidden by Grafana"
                .to_string()
        ));
        assert!(lines.contains(&"Secret review required: true (secure fields present)".to_string()));
    }

    #[test]
    fn detail_lines_sort_json_data_and_secure_json_field_keys() {
        let item = datasource_item(
            json!({
                "jsonData": {
                    "zulu": true,
                    "alpha": true
                },
                "secureJsonFields": {
                    "httpHeaderValue1": true,
                    "basicAuthPassword": true
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = detail_lines(&item);

        assert!(lines.contains(&"jsonData keys: alpha, zulu".to_string()));
        assert!(
            lines.contains(&"secureJsonFields: basicAuthPassword, httpHeaderValue1".to_string())
        );
    }

    #[test]
    fn detail_lines_render_placeholder_backed_secret_review_without_raw_tokens() {
        let item = datasource_item(
            json!({
                "secureJsonDataPlaceholders": {
                    "httpHeaderValue1": "${secret:prom-header}",
                    "basicAuthPassword": "${secret:prom-basic-auth}"
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = detail_lines(&item);
        let rendered = lines.join("\n");

        assert!(lines.contains(
            &"Secret placeholders: available (2 field(s): basicAuthPassword, httpHeaderValue1)"
                .to_string()
        ));
        assert!(
            lines.contains(&"Secret placeholder names: prom-basic-auth, prom-header".to_string())
        );
        assert!(lines.contains(
            &"Secret blocker status: blocked until --secret-values resolves placeholders"
                .to_string()
        ));
        assert!(lines.contains(
            &"Secret review required: true (placeholder-backed secureJsonData)".to_string()
        ));
        assert!(!rendered.contains("${secret:"));
    }

    #[test]
    fn detail_lines_do_not_display_resolved_secure_json_data_values() {
        let item = datasource_item(
            json!({
                "secureJsonData": {
                    "password": "super-secret-value"
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = detail_lines(&item);
        let rendered = lines.join("\n");

        assert!(lines.contains(
            &"Secret material: hidden (1 secureJsonData field(s): password)".to_string()
        ));
        assert!(lines.contains(
            &"Secret review required: true (resolved secureJsonData hidden)".to_string()
        ));
        assert!(!rendered.contains("super-secret-value"));
    }
}
