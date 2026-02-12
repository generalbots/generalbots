use super::types::*;
use super::error::ContactsError;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use log::{error, warn};
use std::sync::Arc;
use uuid::Uuid;

#[derive(QueryableByName)]
struct ContactRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    organization_id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    owner_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    first_name: String,
    #[diesel(sql_type = Nullable<Text>)]
    last_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    email: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    phone: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    mobile: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    company: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    job_title: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    department: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    address_line1: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    address_line2: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    city: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    state: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    postal_code: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    country: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    website: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    linkedin: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    twitter: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    notes: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    tags: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    custom_fields: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    source: Option<String>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Bool)]
    is_favorite: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    last_contacted_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    updated_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

pub struct ContactsService {
    pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
}

impl ContactsService {
    pub fn new(
        pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    ) -> Self {
        Self { pool }
    }

    pub async fn create_contact(
        &self,
        organization_id: Uuid,
        owner_id: Option<Uuid>,
        request: CreateContactRequest,
    ) -> Result<Contact, ContactsError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            ContactsError::DatabaseConnection
        })?;

        let id = Uuid::new_v4();
        let tags_json = serde_json::to_string(&request.tags.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
        let custom_fields_json = serde_json::to_string(&request.custom_fields.unwrap_or_default()).unwrap_or_else(|_| "{}".to_string());
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
                error!("Failed to create contact: {e}");
                ContactsError::CreateFailed
            })?;

        if let Some(group_ids) = request.group_ids {
            for group_id in group_ids {
                self.add_contact_to_group_internal(&mut conn, id, group_id)?;
            }
        }

        self.log_activity(
            &mut conn,
            id,
            ActivityType::Created,
            "Contact created".to_string(),
            None,
            owner_id,
        )?;

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
                error!("Failed to get contact: {e}");
                ContactsError::DatabaseConnection
            })?;

        let row = rows.into_iter().next().ok_or(ContactsError::NotFound)?;
        Ok(self.row_to_contact(row))
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

        let mut count_query = diesel::sql_query(&count_sql).bind::<DieselUuid, _>(organization_id).into_boxed();
        let mut list_query = diesel::sql_query(&list_sql).bind::<DieselUuid, _>(organization_id).into_boxed();

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
        let contacts: Vec<Contact> = rows.into_iter().map(|r| self.row_to_contact(r)).collect();

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
        request: UpdateContactRequest,
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
        let custom_fields_json = serde_json::to_string(&custom_fields).unwrap_or_else(|_| "{}".to_string());

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
                error!("Failed to update contact: {e}");
                ContactsError::UpdateFailed
            })?;

        self.log_activity(
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

        let result = diesel::sql_query(
            "DELETE FROM contacts WHERE id = $1 AND organization_id = $2",
        )
        .bind::<DieselUuid, _>(contact_id)
        .bind::<DieselUuid, _>(organization_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to delete contact: {e}");
            ContactsError::DeleteFailed
        })?;

        if result == 0 {
            return Err(ContactsError::NotFound);
        }

        log::info!("Deleted contact {}", contact_id);
        Ok(())
    }

    pub async fn import_contacts(
        &self,
        organization_id: Uuid,
        owner_id: Option<Uuid>,
        request: ImportRequest,
    ) -> Result<ImportResult, ContactsError> {
        let mut imported_count = 0;
        let mut skipped_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();
        let mut contact_ids = Vec::new();

        match request.format {
            ImportFormat::Csv => {
                let lines: Vec<&str> = request.data.lines().collect();
                if lines.is_empty() {
                    return Ok(ImportResult {
                        success: true,
                        imported_count: 0,
                        skipped_count: 0,
                        error_count: 0,
                        errors: vec![],
                        contact_ids: vec![],
                    });
                }

                let headers: Vec<&str> = lines[0].split(',').map(|s| s.trim()).collect();

                for (line_num, line) in lines.iter().skip(1).enumerate() {
                    let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

                    if values.len() != headers.len() {
                        errors.push(ImportError {
                            line: (line_num + 2) as i32,
                            field: None,
                            message: "Column count mismatch".to_string(),
                        });
                        error_count += 1;
                        continue;
                    }

                    let mut first_name = String::new();
                    let mut last_name = None;
                    let mut email = None;
                    let mut phone = None;
                    let mut company = None;

                    for (i, header) in headers.iter().enumerate() {
                        let value = values.get(i).map(|s| s.to_string());
                        match header.to_lowercase().as_str() {
                            "first_name" | "firstname" | "first name" => {
                                first_name = value.unwrap_or_default();
                            }
                            "last_name" | "lastname" | "last name" => last_name = value,
                            "email" | "e-mail" => email = value,
                            "phone" | "telephone" => phone = value,
                            "company" | "organization" => company = value,
                            _ => {}
                        }
                    }

                    if first_name.is_empty() {
                        errors.push(ImportError {
                            line: (line_num + 2) as i32,
                            field: Some("first_name".to_string()),
                            message: "First name is required".to_string(),
                        });
                        error_count += 1;
                        continue;
                    }

                    if request.skip_duplicates.unwrap_or(true) {
                        if let Some(ref em) = email {
                            if self.email_exists(organization_id, em).await? {
                                skipped_count += 1;
                                continue;
                            }
                        }
                    }

                    let create_req = CreateContactRequest {
                        first_name,
                        last_name,
                        email,
                        phone,
                        mobile: None,
                        company,
                        job_title: None,
                        department: None,
                        address_line1: None,
                        address_line2: None,
                        city: None,
                        state: None,
                        postal_code: None,
                        country: None,
                        website: None,
                        linkedin: None,
                        twitter: None,
                        notes: None,
                        tags: None,
                        custom_fields: None,
                        source: Some(ContactSource::Import),
                        status: None,
                        group_ids: request.group_id.map(|g| vec![g]),
                    };

                    match self.create_contact(organization_id, owner_id, create_req).await {
                        Ok(contact) => {
                            contact_ids.push(contact.id);
                            imported_count += 1;
                        }
                        Err(e) => {
                            errors.push(ImportError {
                                line: (line_num + 2) as i32,
                                field: None,
                                message: e.to_string(),
                            });
                            error_count += 1;
                        }
                    }
                }
            }
            ImportFormat::Vcard => {
                let vcards: Vec<&str> = request.data.split("END:VCARD").collect();

                for (idx, vcard) in vcards.iter().enumerate() {
                    if !vcard.contains("BEGIN:VCARD") {
                        continue;
                    }

                    let mut first_name = String::new();
                    let mut last_name = None;
                    let mut email = None;
                    let mut phone = None;

                    for line in vcard.lines() {
                        if line.starts_with("N:") || line.starts_with("N;") {
                            let parts: Vec<&str> = line.split(':').nth(1).unwrap_or("").split(';').collect();
                            last_name = parts.first().filter(|s| !s.is_empty()).map(|s| s.to_string());
                            first_name = parts.get(1).unwrap_or(&"").to_string();
                        } else if line.starts_with("EMAIL") {
                            email = line.split(':').nth(1).map(|s| s.to_string());
                        } else if line.starts_with("TEL") {
                            phone = line.split(':').nth(1).map(|s| s.to_string());
                        }
                    }

                    if first_name.is_empty() {
                        errors.push(ImportError {
                            line: (idx + 1) as i32,
                            field: Some("first_name".to_string()),
                            message: "First name is required".to_string(),
                        });
                        error_count += 1;
                        continue;
                    }

                    let create_req = CreateContactRequest {
                        first_name,
                        last_name,
                        email,
                        phone,
                        mobile: None,
                        company: None,
                        job_title: None,
                        department: None,
                        address_line1: None,
                        address_line2: None,
                        city: None,
                        state: None,
                        postal_code: None,
                        country: None,
                        website: None,
                        linkedin: None,
                        twitter: None,
                        notes: None,
                        tags: None,
                        custom_fields: None,
                        source: Some(ContactSource::Import),
                        status: None,
                        group_ids: request.group_id.map(|g| vec![g]),
                    };

                    match self.create_contact(organization_id, owner_id, create_req).await {
                        Ok(contact) => {
                            contact_ids.push(contact.id);
                            imported_count += 1;
                        }
                        Err(e) => {
                            errors.push(ImportError {
                                line: (idx + 1) as i32,
                                field: None,
                                message: e.to_string(),
                            });
                            error_count += 1;
                        }
                    }
                }
            }
            ImportFormat::Json => {
                let contacts: Vec<CreateContactRequest> = serde_json::from_str(&request.data)
                    .map_err(|e| ContactsError::ImportFailed(e.to_string()))?;

                for (idx, create_req) in contacts.into_iter().enumerate() {
                    match self.create_contact(organization_id, owner_id, create_req).await {
                        Ok(contact) => {
                            contact_ids.push(contact.id);
                            imported_count += 1;
                        }
                        Err(e) => {
                            errors.push(ImportError {
                                line: (idx + 1) as i32,
                                field: None,
                                message: e.to_string(),
                            });
                            error_count += 1;
                        }
                    }
                }
            }
        }

        log::info!(
            "Import completed: {} imported, {} skipped, {} errors",
            imported_count, skipped_count, error_count
        );

        Ok(ImportResult {
            success: error_count == 0,
            imported_count,
            skipped_count,
            error_count,
            errors,
            contact_ids,
        })
    }

    pub async fn export_contacts(
        &self,
        organization_id: Uuid,
        request: ExportRequest,
    ) -> Result<ExportResult, ContactsError> {
        let contacts = if let Some(ids) = request.contact_ids {
            let mut result = Vec::new();
            for id in ids {
                if let Ok(contact) = self.get_contact(organization_id, id).await {
                    result.push(contact);
                }
            }
            result
        } else {
            let query = ContactListQuery {
                search: None,
                status: None,
                group_id: request.group_id,
                tag: None,
                is_favorite: None,
                sort_by: None,
                sort_order: None,
                page: Some(1),
                per_page: Some(10000),
            };
            self.list_contacts(organization_id, query).await?.contacts
        };

        let contact_count = contacts.len() as i32;

        let (data, content_type, filename) = match request.format {
            ExportFormat::Csv => {
                let mut csv = String::from("first_name,last_name,email,phone,company,job_title,notes\n");
                for c in &contacts {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        c.first_name,
                        c.last_name.as_deref().unwrap_or(""),
                        c.email.as_deref().unwrap_or(""),
                        c.phone.as_deref().unwrap_or(""),
                        c.company.as_deref().unwrap_or(""),
                        c.job_title.as_deref().unwrap_or(""),
                        c.notes.as_deref().unwrap_or("").replace(',', ";")
                    ));
                }
                (csv, "text/csv".to_string(), "contacts.csv".to_string())
            }
            ExportFormat::Vcard => {
                let mut vcf = String::new();
                for c in &contacts {
                    vcf.push_str("BEGIN:VCARD\n");
                    vcf.push_str("VERSION:3.0\n");
                    vcf.push_str(&format!(
                        "N:{};{};;;\n",
                        c.last_name.as_deref().unwrap_or(""),
                        c.first_name
                    ));
                    vcf.push_str(&format!(
                        "FN:{} {}\n",
                        c.first_name,
                        c.last_name.as_deref().unwrap_or("")
                    ));
                    if let Some(ref email) = c.email {
                        vcf.push_str(&format!("EMAIL:{email}\n"));
                    }
                    if let Some(ref phone) = c.phone {
                        vcf.push_str(&format!("TEL:{phone}\n"));
                    }
                    if let Some(ref company) = c.company {
                        vcf.push_str(&format!("ORG:{company}\n"));
                    }
                    vcf.push_str("END:VCARD\n");
                }
                (vcf, "text/vcard".to_string(), "contacts.vcf".to_string())
            }
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&contacts)
                    .map_err(|e| ContactsError::ExportFailed(e.to_string()))?;
                (json, "application/json".to_string(), "contacts.json".to_string())
            }
        };

        Ok(ExportResult {
            success: true,
            data,
            content_type,
            filename,
            contact_count,
        })
    }

    async fn email_exists(&self, organization_id: Uuid, email: &str) -> Result<bool, ContactsError> {
        let mut conn = self.pool.get().map_err(|_| ContactsError::DatabaseConnection)?;

        let result: Vec<CountRow> = diesel::sql_query(
            "SELECT COUNT(*) as count FROM contacts WHERE organization_id = $1 AND email = $2"
        )
        .bind::<DieselUuid, _>(organization_id)
        .bind::<Text, _>(email)
        .load(&mut conn)
        .unwrap_or_default();

        Ok(result.first().map(|r| r.count > 0).unwrap_or(false))
    }

    fn add_contact_to_group_internal(
        &self,
        conn: &mut diesel::PgConnection,
        contact_id: Uuid,
        group_id: Uuid,
    ) -> Result<(), ContactsError> {
        diesel::sql_query(
            "INSERT INTO contact_group_members (contact_id, group_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind::<DieselUuid, _>(contact_id)
        .bind::<DieselUuid, _>(group_id)
        .execute(conn)
        .map_err(|e| {
            error!("Failed to add contact to group: {e}");
            ContactsError::UpdateFailed
        })?;
        Ok(())
    }

    fn log_activity(
        &self,
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
            "#
        )
        .bind::<DieselUuid, _>(id)
        .bind::<DieselUuid, _>(contact_id)
        .bind::<Text, _>(activity_type.to_string())
        .bind::<Text, _>(&title)
        .bind::<Nullable<Text>, _>(description.as_deref())
        .bind::<Nullable<DieselUuid>, _>(performed_by)
        .execute(conn)
        .map_err(|e| {
            warn!("Failed to log activity: {e}");
            ContactsError::UpdateFailed
        })?;
        Ok(())
    }

    fn row_to_contact(&self, row: ContactRow) -> Contact {
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
}
