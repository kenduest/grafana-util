use serde_json::{Map, Value};
use std::collections::BTreeSet;

use crate::common::{message, Result};
use crate::http::JsonHttpClient;
use crate::project_status::{
    ProjectDomainStatus, PROJECT_STATUS_BLOCKED, PROJECT_STATUS_PARTIAL, PROJECT_STATUS_READY,
};
use crate::project_status_model::{merge_status_record_counts, StatusReading, StatusRecordCount};
use crate::project_status_support::build_live_project_status_client_from_api;

use super::{
    build_live_overall_freshness, PROJECT_STATUS_LIVE_ALL_ORGS_AGGREGATE,
    PROJECT_STATUS_LIVE_ALL_ORGS_MODE_SUFFIX,
};

fn project_status_severity_rank(status: &str) -> usize {
    match status {
        PROJECT_STATUS_BLOCKED => 0,
        PROJECT_STATUS_PARTIAL => 1,
        PROJECT_STATUS_READY => 2,
        _ => 3,
    }
}

fn org_id_from_record(org: &Map<String, Value>) -> Result<i64> {
    org.get("id")
        .and_then(Value::as_i64)
        .ok_or_else(|| message("Grafana org payload did not include a usable numeric id."))
}

pub(super) struct ScopedLiveOrgClient {
    #[cfg(test)]
    org_id: i64,
    client: JsonHttpClient,
}

impl ScopedLiveOrgClient {
    #[cfg(test)]
    pub(super) fn org_id(&self) -> i64 {
        self.org_id
    }

    fn client(&self) -> &JsonHttpClient {
        &self.client
    }
}

pub(super) fn build_scoped_live_org_clients(
    api: &crate::grafana_api::GrafanaApiClient,
    orgs: &[Map<String, Value>],
) -> Result<Vec<ScopedLiveOrgClient>> {
    orgs.iter()
        .map(|org| {
            let org_id = org_id_from_record(org)?;
            let client = build_live_project_status_client_from_api(api, Some(org_id))?;
            Ok(ScopedLiveOrgClient {
                #[cfg(test)]
                org_id,
                client,
            })
        })
        .collect()
}

fn merge_live_domain_statuses(statuses: Vec<ProjectDomainStatus>) -> Result<ProjectDomainStatus> {
    let freshness = build_live_overall_freshness(&statuses);
    let aggregate_index = statuses
        .iter()
        .enumerate()
        .min_by_key(|(_, status)| {
            (
                project_status_severity_rank(&status.status),
                usize::MAX - status.blocker_count,
                usize::MAX - status.warning_count,
            )
        })
        .map(|(index, _)| index)
        .ok_or_else(|| message("Expected at least one per-org domain status to aggregate."))?;
    let all_same_reason_code = statuses
        .iter()
        .all(|status| status.reason_code == statuses[aggregate_index].reason_code);
    let all_same_mode = statuses
        .iter()
        .all(|status| status.mode == statuses[aggregate_index].mode);
    let mut primary_count = 0usize;
    let mut source_kinds = BTreeSet::new();
    let mut signal_keys = BTreeSet::new();
    let mut blocker_records = Vec::<StatusRecordCount>::new();
    let mut warning_records = Vec::<StatusRecordCount>::new();
    let mut next_actions = Vec::<String>::new();
    let mut aggregate = None;

    for (index, status) in statuses.into_iter().enumerate() {
        let ProjectDomainStatus {
            id,
            scope,
            mode,
            status,
            reason_code,
            primary_count: status_primary_count,
            blocker_count: _,
            warning_count: _,
            source_kinds: status_source_kinds,
            signal_keys: status_signal_keys,
            blockers: status_blockers,
            warnings: status_warnings,
            next_actions: status_next_actions,
            freshness: _,
        } = status;

        primary_count += status_primary_count;
        source_kinds.extend(status_source_kinds);
        signal_keys.extend(status_signal_keys);
        blocker_records.extend(status_blockers.into_iter().map(StatusRecordCount::from));
        warning_records.extend(status_warnings.into_iter().map(StatusRecordCount::from));
        for action in status_next_actions {
            if !next_actions.iter().any(|existing| existing == &action) {
                next_actions.push(action);
            }
        }

        if index == aggregate_index {
            aggregate = Some((id, scope, mode, status, reason_code));
        }
    }

    let (id, scope, aggregate_mode, aggregate_status, aggregate_reason_code) =
        aggregate.ok_or_else(|| message("Expected aggregate domain status to be available."))?;

    let blockers = merge_status_record_counts(blocker_records);
    let warnings = merge_status_record_counts(warning_records);
    let reason_code = if all_same_reason_code {
        aggregate_reason_code
    } else {
        PROJECT_STATUS_LIVE_ALL_ORGS_AGGREGATE.to_string()
    };
    let mode = if all_same_mode {
        format!(
            "{}{}",
            aggregate_mode, PROJECT_STATUS_LIVE_ALL_ORGS_MODE_SUFFIX
        )
    } else {
        PROJECT_STATUS_LIVE_ALL_ORGS_AGGREGATE.to_string()
    };

    Ok(StatusReading {
        id,
        scope,
        mode,
        status: aggregate_status,
        reason_code,
        primary_count,
        source_kinds: source_kinds.into_iter().collect(),
        signal_keys: signal_keys.into_iter().collect(),
        blockers,
        warnings,
        next_actions,
        freshness,
    }
    .into_project_domain_status())
}

#[cfg(test)]
pub(super) fn build_live_multi_org_domain_status_with_orgs<F>(
    orgs: &[Map<String, Value>],
    mut build_org_status: F,
) -> Result<ProjectDomainStatus>
where
    F: FnMut(i64) -> Result<ProjectDomainStatus>,
{
    let mut statuses = Vec::new();
    for org in orgs {
        statuses.push(build_org_status(org_id_from_record(org)?)?);
    }
    merge_live_domain_statuses(statuses)
}

pub(super) fn build_live_multi_org_domain_status<F>(
    clients: &[ScopedLiveOrgClient],
    mut build_status: F,
) -> Result<ProjectDomainStatus>
where
    F: FnMut(&JsonHttpClient) -> ProjectDomainStatus,
{
    let mut statuses = Vec::new();
    for client in clients {
        statuses.push(build_status(client.client()));
    }
    merge_live_domain_statuses(statuses)
}

pub(super) fn build_live_multi_org_domain_status_pair<F>(
    clients: &[ScopedLiveOrgClient],
    mut build_statuses: F,
) -> Result<(ProjectDomainStatus, ProjectDomainStatus)>
where
    F: FnMut(&JsonHttpClient) -> (ProjectDomainStatus, ProjectDomainStatus),
{
    let mut first_statuses = Vec::new();
    let mut second_statuses = Vec::new();
    for client in clients {
        let (first, second) = build_statuses(client.client());
        first_statuses.push(first);
        second_statuses.push(second);
    }
    Ok((
        merge_live_domain_statuses(first_statuses)?,
        merge_live_domain_statuses(second_statuses)?,
    ))
}

#[cfg(test)]
pub(super) fn build_live_multi_org_domain_status_with_clients<F>(
    clients: &[ScopedLiveOrgClient],
    mut build_status: F,
) -> Result<ProjectDomainStatus>
where
    F: FnMut(&ScopedLiveOrgClient) -> ProjectDomainStatus,
{
    let mut statuses = Vec::new();
    for client in clients {
        statuses.push(build_status(client));
    }
    merge_live_domain_statuses(statuses)
}

#[cfg(test)]
pub(super) fn scoped_live_org_client_for_test(
    org_id: i64,
    client: JsonHttpClient,
) -> ScopedLiveOrgClient {
    ScopedLiveOrgClient { org_id, client }
}
