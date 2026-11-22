//! #2548 — branch-scoped readers for the attribution bridge.
//!
//! Reads only tables that already exist (`crm_contacts`, `crm_deals`,
//! `marketing_recipients`, `marketing_campaign_events`,
//! `marketing_suppressions`, `marketing_list_contacts`, `marketing_lists`,
//! `crm_deal_segments`). Every touch carries the table it came from; no row
//! is rewritten and no migration is needed.

use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::PgConnection;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use botschema::{
    crm_deal_segments, marketing_campaign_events, marketing_list_contacts, marketing_lists,
    marketing_recipients, marketing_suppressions,
};

use super::evaluate::{apply_suppressions, event_touch, normalize_email, recipient_touches};
use super::model::{
    AttributionInput, AttributionTouch, CampaignEventFacts, ContactRevenueBridge,
    InfluencedDeal, MembershipSource, RecipientFacts, SegmentMembership, SuppressionRecord,
    SuppressionSource,
};
use crate::schema::{crm_contacts, crm_deals};
use crate::{ContactsError, CrateState};

type RecipientRow = (
    Uuid,
    Option<Uuid>,
    Option<Uuid>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    Option<Uuid>,
    Option<Uuid>,
);

type DealRow = (
    Uuid,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<f64>,
    Option<String>,
    Option<bool>,
    Option<Uuid>,
    Option<Uuid>,
);

fn scope(state: &CrateState, headers: &HeaderMap, conn: &mut PgConnection) -> Uuid {
    crate::scope::branch_from_jwt(headers, conn).unwrap_or_else(|| state.get_bot_context())
}

fn log_query(error: &diesel::result::Error, table: &str) -> ContactsError {
    log::error!("revenue attribution read failed on {table}: {error}");
    ContactsError::QueryFailed
}

fn emails_to_contacts(rows: Vec<(Uuid, Option<String>)>) -> HashMap<String, Uuid> {
    let mut map = HashMap::new();
    for (id, email) in rows {
        if let Some(value) = email {
            let normalized = normalize_email(&value);
            if !normalized.is_empty() {
                map.insert(normalized, id);
            }
        }
    }
    map
}

fn resolve_contact(
    explicit: Option<Uuid>,
    email: Option<&String>,
    emails: &HashMap<String, Uuid>,
    contact_ids: &HashSet<Uuid>,
) -> Option<Uuid> {
    // A recipient row may carry a stale or cross-tenant contact_id; only a
    // contact that exists in this branch may be claimed as attributed.
    if let Some(id) = explicit {
        if contact_ids.contains(&id) {
            return Some(id);
        }
    }
    email
        .and_then(|value| emails.get(&normalize_email(value)).copied())
        .filter(|id| contact_ids.contains(id))
}

fn segment_names(
    conn: &mut PgConnection,
    branch_id: Uuid,
) -> Result<HashMap<Uuid, String>, ContactsError> {
    let mut map = HashMap::new();
    let deal_rows = crm_deal_segments::table
        .filter(crm_deal_segments::branch_id.eq(branch_id))
        .select((crm_deal_segments::id, crm_deal_segments::name))
        .load::<(Uuid, String)>(conn)
        .map_err(|error| log_query(&error, "crm_deal_segments"))?;
    for (id, name) in deal_rows {
        map.insert(id, name);
    }
    let list_rows = marketing_lists::table
        .filter(marketing_lists::branch_id.eq(branch_id))
        .select((marketing_lists::id, marketing_lists::name))
        .load::<(Uuid, String)>(conn)
        .map_err(|error| log_query(&error, "marketing_lists"))?;
    for (id, name) in list_rows {
        map.insert(id, name);
    }
    Ok(map)
}

/// Load all evidence in the current branch. This is the single audited path
/// used by the aggregate report and by every per-contact view.
pub fn load_input(
    state: &CrateState,
    headers: &HeaderMap,
    campaign_id: Option<Uuid>,
    segment_id: Option<Uuid>,
) -> Result<AttributionInput, ContactsError> {
    let mut conn = state.db_pool.get().map_err(|error| {
        log::error!("revenue attribution pool error: {error}");
        ContactsError::DatabaseConnection
    })?;
    let branch_id = scope(state, headers, &mut conn);

    let contacts_scanned = crm_contacts::table
        .filter(crm_contacts::branch_id.eq(branch_id))
        .count()
        .get_result::<i64>(&mut conn)
        .map_err(|error| log_query(&error, "crm_contacts"))?;
    let email_rows = crm_contacts::table
        .filter(crm_contacts::branch_id.eq(branch_id))
        .select((crm_contacts::id, crm_contacts::email))
        .load::<(Uuid, Option<String>)>(&mut conn)
        .map_err(|error| log_query(&error, "crm_contacts"))?;
    let contact_ids: HashSet<Uuid> = email_rows.iter().map(|(id, _)| *id).collect();
    let emails = emails_to_contacts(email_rows);

    let mut recipient_query = marketing_recipients::table
        .filter(marketing_recipients::branch_id.eq(branch_id))
        .into_boxed();
    if let Some(id) = campaign_id {
        recipient_query = recipient_query.filter(marketing_recipients::campaign_id.eq(id));
    }
    let recipient_rows = recipient_query
        .select((
            marketing_recipients::id,
            marketing_recipients::campaign_id,
            marketing_recipients::contact_id,
            marketing_recipients::email,
            marketing_recipients::channel,
            marketing_recipients::status,
            marketing_recipients::sent_at,
            marketing_recipients::delivered_at,
            marketing_recipients::opened_at,
            marketing_recipients::clicked_at,
            marketing_recipients::created_at,
            marketing_recipients::list_id,
            marketing_recipients::deal_id,
        ))
        .load::<RecipientRow>(&mut conn)
        .map_err(|error| log_query(&error, "marketing_recipients"))?;
    let recipients_scanned = recipient_rows.len() as i64;
    let mut deal_ids: Vec<Uuid> = Vec::new();
    let mut deal_campaigns: HashMap<Uuid, Uuid> = HashMap::new();
    let mut segment_ids: Vec<Uuid> = Vec::new();
    let mut touches: Vec<AttributionTouch> = Vec::new();

    for row in recipient_rows {
        let (
            id,
            row_campaign,
            row_contact,
            email,
            channel,
            status,
            sent_at,
            delivered_at,
            opened_at,
            clicked_at,
            created_at,
            list_id,
            deal_id,
        ) = row;
        if let Some(id) = deal_id {
            if !deal_ids.contains(&id) {
                deal_ids.push(id);
            }
            if let Some(campaign) = row_campaign {
                deal_campaigns.insert(id, campaign);
            }
        }
        if let Some(id) = list_id {
            if !segment_ids.contains(&id) {
                segment_ids.push(id);
            }
        }
        let facts = RecipientFacts {
            id,
            campaign_id: row_campaign,
            contact_id: resolve_contact(row_contact, email.as_ref(), &emails, &contact_ids),
            email,
            channel,
            status,
            sent_at,
            delivered_at,
            opened_at,
            clicked_at,
            occurred_at: created_at.unwrap_or_else(Utc::now),
        };
        touches.extend(recipient_touches(&facts));
    }

    let mut event_query = marketing_campaign_events::table
        .filter(marketing_campaign_events::branch_id.eq(branch_id))
        .into_boxed();
    if let Some(id) = campaign_id {
        event_query = event_query.filter(marketing_campaign_events::campaign_id.eq(id));
    }
    let event_rows = event_query
        .select((
            marketing_campaign_events::id,
            marketing_campaign_events::campaign_id,
            marketing_campaign_events::channel,
            marketing_campaign_events::event_type,
            marketing_campaign_events::recipient_email,
            marketing_campaign_events::created_at,
        ))
        .load::<(Uuid, Uuid, Option<String>, String, Option<String>, DateTime<Utc>)>(&mut conn)
        .map_err(|error| log_query(&error, "marketing_campaign_events"))?;
    for (id, row_campaign, channel, event_type, email, occurred_at) in event_rows {
        touches.push(event_touch(&CampaignEventFacts {
            id,
            campaign_id: Some(row_campaign),
            channel,
            event_type,
            contact_id: resolve_contact(None, email.as_ref(), &emails, &contact_ids),
            recipient_email: email,
            occurred_at,
        }));
    }

    let mut suppressions: Vec<SuppressionRecord> = Vec::new();
    let suppression_rows = marketing_suppressions::table
        .filter(marketing_suppressions::branch_id.eq(branch_id))
        .select((
            marketing_suppressions::email,
            marketing_suppressions::channel,
            marketing_suppressions::reason,
        ))
        .load::<(String, String, String)>(&mut conn)
        .map_err(|error| log_query(&error, "marketing_suppressions"))?;
    for (address, channel, reason) in suppression_rows {
        suppressions.push(SuppressionRecord {
            contact_id: emails.get(&normalize_email(&address)).copied(),
            address,
            channel,
            reason,
            source: SuppressionSource::DurableList,
        });
    }
    for item in &touches {
        if item.kind.is_suppression() {
            let address = item.email.clone().unwrap_or_else(|| "unmatched".to_string());
            suppressions.push(SuppressionRecord {
                contact_id: item.contact_id,
                address,
                channel: item.channel.clone().unwrap_or_else(|| "unknown".to_string()),
                reason: item.raw_event_type.clone(),
                source: SuppressionSource::CampaignEvent,
            });
        }
    }

    let names = segment_names(&mut conn, branch_id)?;
    let mut deal_query = crm_deals::table
        .filter(crm_deals::branch_id.eq(branch_id))
        .into_boxed();
    deal_query = match segment_id {
        Some(id) => deal_query
            .filter(crm_deals::segment_id.eq(id).or(crm_deals::id.eq_any(&deal_ids))),
        None => deal_query
            .filter(crm_deals::segment_id.is_not_null().or(crm_deals::id.eq_any(&deal_ids))),
    };
    let deal_rows = deal_query
        .select((
            crm_deals::id,
            crm_deals::name,
            crm_deals::title,
            crm_deals::stage,
            crm_deals::value,
            crm_deals::currency,
            crm_deals::won,
            crm_deals::segment_id,
            crm_deals::contact_id,
        ))
        .load::<DealRow>(&mut conn)
        .map_err(|error| log_query(&error, "crm_deals"))?;
    let mut deals: Vec<InfluencedDeal> = Vec::new();
    for row in deal_rows {
        let (
            id,
            name,
            title,
            stage,
            value,
            currency,
            won,
            deal_segment,
            contact,
        ) = row;
        if let Some(id) = deal_segment {
            if !segment_ids.contains(&id) {
                segment_ids.push(id);
            }
        }
        deals.push(InfluencedDeal {
            id,
            name: name.or(title),
            stage,
            value,
            currency,
            won: won == Some(true),
            segment_id: deal_segment,
            segment_name: deal_segment.and_then(|id| names.get(&id).cloned()),
            campaign_id: deal_campaigns.get(&id).copied(),
            contact_id: contact,
        });
    }

    // `marketing_list_contacts` has no branch column: restrict by the list ids
    // that belong to this branch before reading membership rows.
    let list_ids: Vec<Uuid> = marketing_lists::table
        .filter(marketing_lists::branch_id.eq(branch_id))
        .select(marketing_lists::id)
        .load::<Uuid>(&mut conn)
        .map_err(|error| log_query(&error, "marketing_lists"))?;
    let mut memberships: Vec<SegmentMembership> = Vec::new();
    let membership_rows = marketing_list_contacts::table
        .filter(marketing_list_contacts::list_id.eq_any(&list_ids))
        .select((marketing_list_contacts::list_id, marketing_list_contacts::contact_id))
        .load::<(Uuid, Uuid)>(&mut conn)
        .map_err(|error| log_query(&error, "marketing_list_contacts"))?;
    for (list_id, contact_id) in membership_rows {
        if !contact_ids.contains(&contact_id) {
            continue;
        }
        if !segment_ids.contains(&list_id) {
            segment_ids.push(list_id);
        }
        memberships.push(SegmentMembership {
            segment_id: list_id,
            segment_name: names.get(&list_id).cloned(),
            source: MembershipSource::AudienceList,
            contact_id,
            suppressed_channels: Vec::new(),
        });
    }
    let memberships = apply_suppressions(memberships, &suppressions);

    Ok(AttributionInput {
        branch_id,
        campaign_id,
        segment_id,
        contacts_scanned,
        recipients_scanned,
        touches,
        suppressions,
        deals,
        segment_ids,
        memberships,
    })
}

/// Per-contact view. Reuses the aggregate loader so a contact can never be
/// scored by a different set of rules than the report it contributes to.
pub fn load_contact_bridge(
    state: &CrateState,
    headers: &HeaderMap,
    contact_id: Uuid,
) -> Result<Option<ContactRevenueBridge>, ContactsError> {
    let input = load_input(state, headers, None, None)?;
    let mut conn = state.db_pool.get().map_err(|error| {
        log::error!("revenue attribution pool error: {error}");
        ContactsError::DatabaseConnection
    })?;
    let branch_id = scope(state, headers, &mut conn);
    let emails = crm_contacts::table
        .filter(crm_contacts::branch_id.eq(branch_id))
        .filter(crm_contacts::id.eq(contact_id))
        .select(crm_contacts::email)
        .load::<Option<String>>(&mut conn)
        .map_err(|error| log_query(&error, "crm_contacts"))?;
    let email = match emails.into_iter().next() {
        Some(value) => value,
        None => return Ok(None),
    };

    let touches: Vec<AttributionTouch> = input
        .touches
        .into_iter()
        .filter(|item| item.contact_id == Some(contact_id))
        .collect();
    let suppressions: Vec<SuppressionRecord> = input
        .suppressions
        .into_iter()
        .filter(|item| item.contact_id == Some(contact_id))
        .collect();
    let memberships: Vec<SegmentMembership> = input
        .memberships
        .into_iter()
        .filter(|item| item.contact_id == contact_id)
        .collect();
    let deals: Vec<InfluencedDeal> = input
        .deals
        .into_iter()
        .filter(|item| item.contact_id == Some(contact_id))
        .collect();

    Ok(Some(ContactRevenueBridge {
        contact_id,
        email,
        touches,
        memberships,
        suppressions,
        deals,
    }))
}
