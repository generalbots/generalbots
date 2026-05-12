use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable, Text, Uuid as DieselUuid};
use std::sync::Arc;
use uuid::Uuid;

use crate::contacts_api_helpers::{
    add_contact_to_group_internal, log_activity, row_to_contact, ContactRow, CountRow,
};
use crate::error::ContactsError;
use crate::models::*;
use crate::requests::*;

pub struct ContactsService {
    pool: Arc<crate::DbPool>,
}

impl ContactsService {
    pub fn new(pool: Arc<crate::DbPool>) -> Self {
        Self { pool }
    }

    pub async fn create_contact(
        &self,
        organization_id: Uuid,
        owner_id: Option<Uuid>,
        request: ContactsApiCreateRequest,
    ) -> Result<Contact, ContactsError> {
        let mut conn = self.pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {e}");
            ContactsError::DatabaseConnection
        })?;

        let id = Uuid::new_v4();
        let tags_json =
            serde_json::to_string(&request.tags.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
        let custom_fields_json = serde_json::to_string(&request.custom_fields.unwrap_or_default())
            .unwrap_or_else(|_| "{}".to_string());
        let source_str = request.source.map(|s| s.to_string());
        let status_str = request.status.unwrap_or_default().to_string();

        let sql = r#"
            INSERT INTO contacts (
                id, organization_id, owner_id, first_name, last_name, email, phone, mobile,
                company, job_title, department, address_line1, address_line2, city, state,
                postal_code, country, website, linkedin, twitter, notes, tags, custom_fields,
                source, status, is_favorite, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23, $24, $25, FALSE, NOW(), NOW()
            )
        "#;

        diesel::sql_query(sql)
            .bind::<DieselUuid, _>(id)
            .bind::<DieselUuid, _>(organization_id)
            .bind::<Nullable<DieselUuid>, _>(owner_id)
            .bind::<Text, _>(&request.first_name)
            .bind::<Nullable<Text>, _>(request.last_name.as_deref())
            .bind::<Nullable<Text>, _>(request.email.as_deref())
            .bind::<Nullable<Text>, _>(request.phone.as_deref())
            .bind::<Nullable<Text>, _>(request.mobile.as_deref())
            .bind::<Nullable<Text>, _>(request.company.as_deref())
            .bind::<Nullable<Text>, _>(request.job_title.as_deref())
            .bind::<Nullable<Text>, _>(request.department.as_deref())
            .bind::<Nullable<Text>, _>(request.address_line1.as_deref())
            .bind::<Nullable<Text>, _>(request.address_line2.as_deref())
            .bind::<Nullable<Text>, _>(request.city.as_deref())
            .bind::<Nullable<Text>, _>(request.state.as_deref())
            .bind::<Nullable<Text>, _>(request.postal_code.as_deref())
            .bind::<Nullable<Text>, _>(request.country.as_deref())
            .bind::<Nullable<Text>, _>(request.website.as_deref())
            .bind::<Nullable<Text>, _>(request.linkedin.as_deref())
            .bind::<Nullable<Text>, _>(request.twitter.as_deref())
            .bind::<Nullable<Text>, _>(request.notes.as_deref())
            .bind::<Text, _>(&tags_json)
            .bind::<Text, _>(&custom_fields_json)
            .bind::<Nullable<Text>, _>(source_str.as_deref())
            .bind::<Text, _>(&status_str)
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Failed to create contact: {e}");
                ContactsError::CreateFailed
            })?;

        if let Some(group_ids) = request.group_ids {
            for group_id in group_ids {
                add_contact_to_group_internal(&mut conn, id, group_id)?;
            }
        }

        log_activity(&mut conn, id, ActivityType::Created, "Contact created".to_string(), None, owner_id)?;

        self.get_contact(organization_id, id).await
    }

    pub async fn get_contact(
        &self,
        organization_id: Uuid,
        contact_id: Uuid,
    ) -> Result<Contact, ContactsError> {
        let mut conn = self.pool.get().map_err(|_| ContactsError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, organization_id, owner_id, first_name, last_name, email, phone, mobile,
            company, job_title, department, address_line1, address_line2, city, state,
            postal_code, country, website, linkedin, twitter, notes, tags, custom_fields,
            source, status, is_favorite, last_contacted_at, created_at, updated_at
            FROM contacts
            WHERE id = $1 AND organization_id = $2
        "#;

        let rows: Vec<ContactRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(contact_id)
            .bind::<DieselUuid, _>(organization_id)
            .load(&mut conn)
            .map_err(|e| {
                log::error!("Failed to get contact: {e}");
                ContactsError::DatabaseConnection
            })?;

        let row = rows.into_iter().next().ok_or(ContactsError::NotFound)?;
        Ok(row_to_contact(row))
    }

    pub async fn list_contacts(
        &self,
        organization_id: Uuid,
        query: ContactListQuery,
    ) -> Result<ContactListResponse, ContactsError> {
        let mut conn = self.pool.get().map_err(|_| ContactsError::DatabaseConnection)?;

        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(25).clamp(1, 100);
        let offset = (page - 1) * per_page;

        let mut where_clauses = vec!["organization_id = $1".to_string()];
        let mut param_count = 1;

        if query.search.is_some() {
            param_count += 1;
            where_clauses.push(format!(
                "(first_name ILIKE '%' || ${param_count} || '%' OR last_name ILIKE '%' || ${param_count} || '%' OR email ILIKE '%' || ${param_count} || '%' OR company ILIKE '%' || ${param_count} || '%')"
            ));
        }

        if query.status.is_some() {
            param_count += 1;
            where_clauses.push(format!("status = ${param_count}"));
        }

        if query.is_favorite.is_some() {
            param_count += 1;
            where_clauses.push(format!("is_favorite = ${param_count}"));
        }

        if query.tag.is_some() {
            param_count += 1;
            where_clauses.push(format!("tags::jsonb ? ${param_count}"));
        }

        let where_clause = where_clauses.join(" AND ");

        let sort_column = match query.sort_by.as_deref() {
            Some("first_name") => "first_name",
            Some("last_name") => "last_name",
            Some("email") => "email",
            Some("company") => "company",
            Some("created_at") => "created_at",
            Some("updated_at") => "updated_at",
            Some("last_contacted_at") => "last_contacted_at",
            _ => "created_at",
        };

        let sort_order = match query.sort_order.as_deref() {
            Some("asc") => "ASC",
            _ => "DESC",
        };

        let count_sql = format!("SELECT COUNT(*) as count FROM contacts WHERE {where_clause}");
        let list_sql = format!(
            r#"
            SELECT id, organization_id, owner_id, first_name, last_name, email, phone, mobile,
            company, job_title, department, address_line1, address_line2, city, state,
            postal_code, country, website, linkedin, twitter, notes, tags, custom_fields,
            source, status, is_favorite, last_contacted_at, created_at, updated_at
            FROM contacts
            WHERE {where_clause}
            ORDER BY {sort_column} {sort_order}
            LIMIT ${} OFFSET ${}
            "#,
            param_count + 1,
            param_count + 2
        );

        let mut count_query =
            diesel::sql_query(&count_sql).bind::<DieselUuid, _>(organization_id).into_boxed();
        let mut list_query =
            diesel::sql_query(&list_sql).bind::<DieselUuid, _>(organization_id).into_boxed();

        if let Some(ref search) = query.search {
            count_query = count_query.bind::<Text, _>(search);
            list_query = list_query.bind::<Text, _>(search);
        }

        if let Some(ref status) = query.status {
            count_query = count_query.bind::<Text, _>(status.to_string());
            list_query = list_query.bind::<Text, _>(status.to_string());
        }

        if let Some(is_fav) = query.is_favorite {
            count_query = count_query.bind::<Bool, _>(is_fav);
            list_query = list_query.bind::<Bool, _>(is_fav);
        }

        if let Some(ref tag) = query.tag {
            count_query = count_query.bind::<Text, _>(tag);
            list_query = list_query.bind::<Text, _>(tag);
        }

        list_query = list_query
            .bind::<diesel::sql_types::Integer, _>(per_page)
            .bind::<diesel::sql_types::Integer, _>(offset);

        let count_result: Vec<CountRow> = count_query.load(&mut conn).unwrap_or_default();
        let total_count = count_result.first().map(|r| r.count).unwrap_or(0);

        let rows: Vec<ContactRow> = list_query.load(&mut conn).unwrap_or_default();
        let contacts: Vec<Contact> = rows.into_iter().map(row_to_contact).collect();
        let total_pages = ((total_count as f64) / (per_page as f64)).ceil() as i32;

        Ok(ContactListResponse {
            contacts,
            total_count,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn update_contact(
        &self,
        organization_id: Uuid,
        contact_id: Uuid,
        request: ContactsApiUpdateRequest,
        updated_by: Option<Uuid>,
    ) -> Result<Contact, ContactsError> {
        let mut conn = self.pool.get().map_err(|_| ContactsError::DatabaseConnection)?;
        let existing = self.get_contact(organization_id, contact_id).await?;

        let first_name = request.first_name.unwrap_or(existing.first_name);
        let last_name = request.last_name.or(existing.last_name);
        let email = request.email.or(existing.email);
        let phone = request.phone.or(existing.phone);
        let mobile = request.mobile.or(existing.mobile);
        let company = request.company.or(existing.company);
        let job_title = request.job_title.or(existing.job_title);
        let department = request.department.or(existing.department);
        let address_line1 = request.address_line1.or(existing.address_line1);
        let address_line2 = request.address_line2.or(existing.address_line2);
        let city = request.city.or(existing.city);
        let state = request.state.or(existing.state);
        let postal_code = request.postal_code.or(existing.postal_code);
        let country = request.country.or(existing.country);
        let website = request.website.or(existing.website);
        let linkedin = request.linkedin.or(existing.linkedin);
        let twitter = request.twitter.or(existing.twitter);
        let notes = request.notes.or(existing.notes);
        let tags = request.tags.unwrap_or(existing.tags);
        let custom_fields = request.custom_fields.unwrap_or(existing.custom_fields);
        let status = request.status.unwrap_or(existing.status);
        let is_favorite = request.is_favorite.unwrap_or(existing.is_favorite);

        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        let custom_fields_json =
            serde_json::to_string(&custom_fields).unwrap_or_else(|_| "{}".to_string());

        let sql = r#"
            UPDATE contacts SET
            first_name = $1, last_name = $2, email = $3, phone = $4, mobile = $5,
            company = $6, job_title = $7, department = $8, address_line1 = $9,
            address_line2 = $10, city = $11, state = $12, postal_code = $13, country = $14,
            website = $15, linkedin = $16, twitter = $17, notes = $18, tags = $19,
            custom_fields = $20, status = $21, is_favorite = $22, updated_at = NOW()
            WHERE id = $23 AND organization_id = $24
        "#;

        diesel::sql_query(sql)
            .bind::<Text, _>(&first_name)
            .bind::<Nullable<Text>, _>(last_name.as_deref())
            .bind::<Nullable<Text>, _>(email.as_deref())
            .bind::<Nullable<Text>, _>(phone.as_deref())
            .bind::<Nullable<Text>, _>(mobile.as_deref())
            .bind::<Nullable<Text>, _>(company.as_deref())
            .bind::<Nullable<Text>, _>(job_title.as_deref())
            .bind::<Nullable<Text>, _>(department.as_deref())
            .bind::<Nullable<Text>, _>(address_line1.as_deref())
            .bind::<Nullable<Text>, _>(address_line2.as_deref())
            .bind::<Nullable<Text>, _>(city.as_deref())
            .bind::<Nullable<Text>, _>(state.as_deref())
            .bind::<Nullable<Text>, _>(postal_code.as_deref())
            .bind::<Nullable<Text>, _>(country.as_deref())
            .bind::<Nullable<Text>, _>(website.as_deref())
            .bind::<Nullable<Text>, _>(linkedin.as_deref())
            .bind::<Nullable<Text>, _>(twitter.as_deref())
            .bind::<Nullable<Text>, _>(notes.as_deref())
            .bind::<Text, _>(&tags_json)
            .bind::<Text, _>(&custom_fields_json)
            .bind::<Text, _>(status.to_string())
            .bind::<Bool, _>(is_favorite)
            .bind::<DieselUuid, _>(contact_id)
            .bind::<DieselUuid, _>(organization_id)
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Failed to update contact: {e}");
                ContactsError::UpdateFailed
            })?;

        log_activity(
            &mut conn,
            contact_id,
            ActivityType::Updated,
            "Contact updated".to_string(),
            None,
            updated_by,
        )?;

        self.get_contact(organization_id, contact_id).await
    }

    pub async fn delete_contact(
        &self,
        organization_id: Uuid,
        contact_id: Uuid,
    ) -> Result<(), ContactsError> {
        let mut conn = self.pool.get().map_err(|_| ContactsError::DatabaseConnection)?;

        let result = diesel::sql_query("DELETE FROM contacts WHERE id = $1 AND organization_id = $2")
            .bind::<DieselUuid, _>(contact_id)
            .bind::<DieselUuid, _>(organization_id)
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Failed to delete contact: {e}");
                ContactsError::DeleteFailed
            })?;

        if result == 0 {
            return Err(ContactsError::NotFound);
        }

        log::info!("Deleted contact {}", contact_id);
        Ok(())
    }
}
