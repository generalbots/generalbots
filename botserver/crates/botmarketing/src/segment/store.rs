//! Database-backed candidate loading, preview and idempotent materialization.

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{Connection, PgConnection};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt;
use uuid::Uuid;

use crate::lists::MarketingList;
use crate::schema::{
    crm_contacts, crm_leads, crm_opportunities, marketing_contacts, marketing_list_contacts,
    marketing_lists,
};

use super::evaluate::{
    candidate_matches, consent_flags, is_suppressed, ConsentFlags, SegmentCandidate,
    SUPPRESSED_STATUSES,
};
use super::model::{Segment, SegmentError, SegmentMeta};
use super::validate::{compile_legacy_query, parse_segment, segment_fingerprint};

/// Errors raised by segment storage operations.
#[derive(Debug)]
pub enum SegmentStoreError {
    Validation(SegmentError),
    Database(diesel::result::Error),
    Serialization(serde_json::Error),
}

impl fmt::Display for SegmentStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(error) => write!(formatter, "invalid segment: {error}"),
            Self::Database(_) => write!(formatter, "database error during segment operation"),
            Self::Serialization(_) => write!(formatter, "segment serialization error"),
        }
    }
}

impl std::error::Error for SegmentStoreError {}

impl From<SegmentError> for SegmentStoreError {
    fn from(error: SegmentError) -> Self {
        Self::Validation(error)
    }
}

impl From<diesel::result::Error> for SegmentStoreError {
    fn from(error: diesel::result::Error) -> Self {
        Self::Database(error)
    }
}

impl From<serde_json::Error> for SegmentStoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

type CrmRow = (
    Uuid,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    Option<Vec<Option<String>>>,
    Option<Uuid>,
    DateTime<Utc>,
    Option<Value>,
);

type DealRow = (Option<Uuid>, Option<BigDecimal>, String);

/// Loads every CRM contact in scope with the denormalized values the segment
/// catalog needs. Pure read — no membership rows are written here.
fn load_candidates(
    conn: &mut PgConnection,
    branch_id: Uuid,
) -> Result<Vec<SegmentCandidate>, SegmentStoreError> {
    let rows: Vec<CrmRow> = crm_contacts::table
        .filter(crm_contacts::branch_id.eq(branch_id))
        .select((
            crm_contacts::id,
            crm_contacts::first_name,
            crm_contacts::last_name,
            crm_contacts::email,
            crm_contacts::status,
            crm_contacts::company,
            crm_contacts::tags,
            crm_contacts::owner_id,
            crm_contacts::updated_at,
            crm_contacts::custom_fields,
        ))
        .load::<CrmRow>(conn)?;

    let leads: Vec<DealRow> = crm_leads::table
        .filter(crm_leads::branch_id.eq(branch_id))
        .select((crm_leads::contact_id, crm_leads::value, crm_leads::stage))
        .load::<DealRow>(conn)?;
    let opportunities: Vec<DealRow> = crm_opportunities::table
        .filter(crm_opportunities::branch_id.eq(branch_id))
        .select((
            crm_opportunities::contact_id,
            crm_opportunities::value,
            crm_opportunities::stage,
        ))
        .load::<DealRow>(conn)?;

    let list_ids: Vec<Uuid> = marketing_lists::table
        .filter(marketing_lists::branch_id.eq(branch_id))
        .select(marketing_lists::id)
        .load::<Uuid>(conn)?;
    let memberships: Vec<(Uuid, Uuid)> = marketing_list_contacts::table
        .filter(marketing_list_contacts::list_id.eq_any(list_ids))
        .select((
            marketing_list_contacts::list_id,
            marketing_list_contacts::contact_id,
        ))
        .load::<(Uuid, Uuid)>(conn)?;

    let subscriptions: Vec<(Option<bool>, String)> = marketing_contacts::table
        .filter(marketing_contacts::branch_id.eq(branch_id))
        .select((marketing_contacts::subscribed, marketing_contacts::email))
        .load::<(Option<bool>, String)>(conn)?;

    let mut deals: HashMap<Uuid, (Option<f64>, Vec<String>)> = HashMap::new();
    for (contact_id, value, stage) in leads.into_iter().chain(opportunities) {
        let Some(id) = contact_id else {
            continue;
        };
        let entry = deals.entry(id).or_insert((None, Vec::new()));
        if let Some(number) = value.and_then(|amount| amount.to_f64()) {
            if entry.0.map_or(true, |current| number > current) {
                entry.0 = Some(number);
            }
        }
        entry.1.push(stage);
    }

    let mut lists_by_contact: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for (list_id, contact_id) in memberships {
        lists_by_contact.entry(contact_id).or_default().push(list_id);
    }

    let subscribed_by_email: HashMap<String, Option<bool>> = subscriptions
        .into_iter()
        .map(|(flag, email)| (email.trim().to_lowercase(), flag))
        .collect();

    Ok(rows
        .into_iter()
        .map(|row| {
            let (
                id,
                first_name,
                last_name,
                email,
                status,
                company,
                tags,
                owner_id,
                last_activity,
                custom_fields,
            ) = row;
            let (deal_value, deal_stages) = deals.remove(&id).unwrap_or((None, Vec::new()));
            let normalized_email = email.as_deref().unwrap_or("").trim().to_lowercase();
            SegmentCandidate {
                id,
                first_name,
                last_name,
                email,
                status,
                company,
                tags: tags.into_iter().flatten().collect(),
                owner_id,
                last_activity,
                deal_value,
                deal_stages,
                list_ids: lists_by_contact.get(&id).cloned().unwrap_or_default(),
                consent: consent_flags(custom_fields.as_ref()),
                marketing_subscribed: subscribed_by_email
                    .get(&normalized_email)
                    .copied()
                    .flatten(),
            }
        })
        .collect())
}

/// Number of materialized membership rows currently stored for a list. Used
/// to detect member drift against the count recorded at the last refresh.
pub fn current_member_count(
    conn: &mut PgConnection,
    list_id: Uuid,
) -> Result<i64, diesel::result::Error> {
    marketing_list_contacts::table
        .filter(marketing_list_contacts::list_id.eq(list_id))
        .count()
        .get_result(conn)
}

/// One row of the preview's first-ten sample.
#[derive(Debug, Clone, Serialize)]
pub struct SegmentPreviewMatch {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub status: String,
}

/// Side-effect-free preview: counts and a bounded sample of the contacts that
/// would be included. Suppressed matches are counted but never sampled.
#[derive(Debug, Clone, Serialize)]
pub struct SegmentPreview {
    pub matched_count: i64,
    pub suppressed_count: i64,
    pub first_matches: Vec<SegmentPreviewMatch>,
}

/// Result of an idempotent materialization refresh.
#[derive(Debug, Clone, Serialize)]
pub struct MaterializeResult {
    pub matched_count: i64,
    pub suppressed_count: i64,
    pub member_count: i64,
    pub query_hash: String,
    pub last_refreshed_at: String,
}

/// Resolves the structured segment stored in `criteria`, falling back to the
/// legacy `query_text` heuristic for lists created before #2537.
pub fn segment_from_list(list: &MarketingList) -> Result<Segment, SegmentStoreError> {
    if let Some(value) = list.criteria.as_ref() {
        if !value.is_null() {
            return Ok(serde_json::from_value(value.clone())?);
        }
    }
    Ok(compile_legacy_query(list.query_text.as_deref().unwrap_or("")))
}

/// Counts matches and suppressed matches and returns at most ten samples.
/// Nothing is persisted; this is safe to call repeatedly from the editor.
pub fn preview_segment(
    conn: &mut PgConnection,
    branch_id: Uuid,
    segment: &Segment,
) -> Result<SegmentPreview, SegmentStoreError> {
    let validated = parse_segment(segment)?;
    let candidates = load_candidates(conn, branch_id)?;
    let mut matched_count = 0i64;
    let mut suppressed_count = 0i64;
    let mut first_matches: Vec<SegmentPreviewMatch> = Vec::new();
    for candidate in &candidates {
        if !candidate_matches(candidate, &validated) {
            continue;
        }
        if is_suppressed(candidate) {
            suppressed_count += 1;
            continue;
        }
        matched_count += 1;
        if first_matches.len() < 10 {
            first_matches.push(SegmentPreviewMatch {
                id: candidate.id,
                name: candidate.display_name(),
                email: candidate.email.clone(),
                status: candidate.status.clone(),
            });
        }
    }
    Ok(SegmentPreview {
        matched_count,
        suppressed_count,
        first_matches,
    })
}

/// Replaces the list's membership with the current eligible set in one
/// transaction. Running it twice with the same segment and data is a no-op on
/// the final membership, and the refresh snapshot is written to `criteria`.
pub fn materialize_segment(
    conn: &mut PgConnection,
    branch_id: Uuid,
    list: &MarketingList,
    segment: &Segment,
) -> Result<MaterializeResult, SegmentStoreError> {
    let validated = parse_segment(segment)?;
    let candidates = load_candidates(conn, branch_id)?;
    let mut eligible_ids: Vec<Uuid> = Vec::with_capacity(candidates.len());
    let mut matched_count = 0i64;
    let mut suppressed_count = 0i64;
    for candidate in &candidates {
        if !candidate_matches(candidate, &validated) {
            continue;
        }
        if is_suppressed(candidate) {
            suppressed_count += 1;
        } else {
            matched_count += 1;
            eligible_ids.push(candidate.id);
        }
    }

    let member_count = eligible_ids.len() as i64;
    let now = Utc::now();
    let last_refreshed_at = now.to_rfc3339();
    let query_hash = segment_fingerprint(segment);
    let mut criteria = serde_json::to_value(segment)?;
    if let Some(object) = criteria.as_object_mut() {
        let meta = SegmentMeta {
            last_refreshed_at: Some(last_refreshed_at.clone()),
            member_count: Some(member_count),
            query_hash: Some(query_hash.clone()),
        };
        object.insert("meta".to_string(), serde_json::to_value(meta)?);
    }

    conn.transaction::<(), diesel::result::Error, _>(|conn| {
        diesel::delete(
            marketing_list_contacts::table.filter(marketing_list_contacts::list_id.eq(list.id)),
        )
        .execute(conn)?;
        for contact_id in &eligible_ids {
            diesel::insert_into(marketing_list_contacts::table)
                .values((
                    marketing_list_contacts::list_id.eq(list.id),
                    marketing_list_contacts::contact_id.eq(*contact_id),
                    marketing_list_contacts::added_at.eq(now),
                ))
                .execute(conn)?;
        }
        diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(list.id)))
            .set((
                marketing_lists::member_count.eq(Some(member_count as i32)),
                marketing_lists::contact_count.eq(Some(member_count as i32)),
                marketing_lists::criteria.eq(Some(criteria)),
                marketing_lists::updated_at.eq(Some(now)),
            ))
            .execute(conn)?;
        Ok(())
    })?;

    Ok(MaterializeResult {
        matched_count,
        suppressed_count,
        member_count,
        query_hash,
        last_refreshed_at,
    })
}

/// CRM contacts whose status blocks delivery, keyed by normalized email. Send
/// paths use this to exclude opted-out recipients even when a list still holds
/// stale membership rows.
pub fn suppressed_crm_emails(
    conn: &mut PgConnection,
    branch_id: Uuid,
) -> Result<HashSet<String>, diesel::result::Error> {
    let rows: Vec<Option<String>> = crm_contacts::table
        .filter(crm_contacts::branch_id.eq(branch_id))
        .filter(crm_contacts::status.eq_any(SUPPRESSED_STATUSES.to_vec()))
        .select(crm_contacts::email)
        .load::<Option<String>>(conn)?;
    Ok(rows
        .into_iter()
        .flatten()
        .map(|email| email.trim().to_lowercase())
        .collect())
}

/// Applies both isolated-list consent flags and CRM status suppression to
/// recipients loaded from `marketing_contacts`. Shared by every send path so a
/// stale membership row can never resurrect an opted-out recipient.
pub fn resolve_send_contacts(
    rows: Vec<(Uuid, String, Option<String>, Option<bool>, Option<Value>)>,
    suppressed: &HashSet<String>,
) -> Vec<(Uuid, String, String)> {
    rows.into_iter()
        .filter_map(|(id, email, name, subscribed, metadata)| {
            if suppressed_marketing_contact(subscribed, metadata.as_ref()) {
                return None;
            }
            if suppressed.contains(&email.trim().to_lowercase()) {
                return None;
            }
            Some((id, email, name.unwrap_or_default()))
        })
        .collect()
}

/// Drops CRM-suppressed emails from a recipient list that has no consent
/// columns (rows already linked to a campaign).
pub fn drop_suppressed(
    contacts: Vec<(Uuid, String, String)>,
    suppressed: &HashSet<String>,
) -> Vec<(Uuid, String, String)> {
    contacts
        .into_iter()
        .filter(|(_, email, _)| !suppressed.contains(&email.trim().to_lowercase()))
        .collect()
}

/// Suppression check for an isolated `marketing_contacts` row at send time.
pub fn suppressed_marketing_contact(subscribed: Option<bool>, metadata: Option<&Value>) -> bool {
    if subscribed == Some(false) {
        return true;
    }
    let flags: ConsentFlags = consent_flags(metadata);
    [flags.email, flags.sms, flags.whatsapp]
        .iter()
        .any(|flag| *flag == Some(false))
}
