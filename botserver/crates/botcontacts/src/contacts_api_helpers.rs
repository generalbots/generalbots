use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use std::collections::HashMap;
use uuid::Uuid;

use chrono::{DateTime, Utc};

use crate::error::ContactsError;
use crate::models::*;


#[derive(QueryableByName)]
pub(crate) struct ContactRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    pub organization_id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    pub owner_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    pub first_name: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub last_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub email: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub phone: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub mobile: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub company: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub job_title: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub department: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub address_line1: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub address_line2: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub city: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub state: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub postal_code: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub country: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub website: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub linkedin: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub twitter: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub notes: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub tags: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub custom_fields: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub source: Option<String>,
    #[diesel(sql_type = Text)]
    pub status: String,
    #[diesel(sql_type = Bool)]
    pub is_favorite: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub last_contacted_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
pub(crate) struct CountRow {
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

pub(crate) fn row_to_contact(row: ContactRow) -> Contact {
    let tags: Vec<String> = row
        .tags
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    let custom_fields: HashMap<String, String> = row
        .custom_fields
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();
    let source = row.source.and_then(|s| match s.as_str() {
        "manual" => Some(ContactSource::Manual),
        "import" => Some(ContactSource::Import),
        "web_form" => Some(ContactSource::WebForm),
        "api" => Some(ContactSource::Api),
        "email" => Some(ContactSource::Email),
        "meeting" => Some(ContactSource::Meeting),
        "referral" => Some(ContactSource::Referral),
        "social" => Some(ContactSource::Social),
        _ => None,
    });
    let status = match row.status.as_str() {
        "active" => ContactStatus::Active,
        "inactive" => ContactStatus::Inactive,
        "lead" => ContactStatus::Lead,
        "customer" => ContactStatus::Customer,
        "prospect" => ContactStatus::Prospect,
        "archived" => ContactStatus::Archived,
        _ => ContactStatus::Active,
    };

    Contact {
        id: row.id,
        organization_id: row.organization_id,
        owner_id: row.owner_id,
        first_name: row.first_name,
        last_name: row.last_name,
        email: row.email,
        phone: row.phone,
        mobile: row.mobile,
        company: row.company,
        job_title: row.job_title,
        department: row.department,
        address_line1: row.address_line1,
        address_line2: row.address_line2,
        city: row.city,
        state: row.state,
        postal_code: row.postal_code,
        country: row.country,
        website: row.website,
        linkedin: row.linkedin,
        twitter: row.twitter,
        notes: row.notes,
        tags,
        custom_fields,
        source,
        status,
        is_favorite: row.is_favorite,
        last_contacted_at: row.last_contacted_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

pub(crate) fn log_activity(
    conn: &mut diesel::PgConnection,
    contact_id: Uuid,
    activity_type: ActivityType,
    title: String,
    description: Option<String>,
    performed_by: Option<Uuid>,
) -> Result<(), ContactsError> {
    let id = Uuid::new_v4();
    diesel::sql_query(
        r#"
        INSERT INTO contact_activities (id, contact_id, activity_type, title, description, performed_by, occurred_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
        "#,
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(contact_id)
    .bind::<Text, _>(activity_type.to_string())
    .bind::<Text, _>(&title)
    .bind::<Nullable<Text>, _>(description.as_deref())
    .bind::<Nullable<DieselUuid>, _>(performed_by)
    .execute(conn)
    .map_err(|e| {
        log::warn!("Failed to log activity: {e}");
        ContactsError::UpdateFailed
    })?;
    Ok(())
}

pub(crate) fn add_contact_to_group_internal(
    conn: &mut diesel::PgConnection,
    contact_id: Uuid,
    group_id: Uuid,
) -> Result<(), ContactsError> {
    diesel::sql_query(
        "INSERT INTO contact_group_members (contact_id, group_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind::<DieselUuid, _>(contact_id)
    .bind::<DieselUuid, _>(group_id)
    .execute(conn)
    .map_err(|e| {
        log::error!("Failed to add contact to group: {e}");
        ContactsError::UpdateFailed
    })?;
    Ok(())
}
