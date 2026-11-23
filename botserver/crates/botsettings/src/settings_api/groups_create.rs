use super::*;
use botsecurity::rbac_middleware::is_admin_group_name;

/// #2633 — reject ambiguous group names that merely *contain* `admin` but
/// are not one of the canonical administrator names. Such names used to grant
/// full admin through a substring check; now that role derivation is exact,
/// refusing them at creation time also stops provisioning/admin UIs from
/// reintroducing the confusion in the first place.
fn validate_group_name(name: &str) -> Result<(), &'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Group name is required");
    }
    if trimmed.to_ascii_lowercase().contains("admin") && !is_admin_group_name(trimmed) {
        return Err(
            "Group name uses a reserved administrator substring; use an exact admin role name or a non-admin name",
        );
    }
    Ok(())
}

pub async fn groups_create(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(form): axum::extract::Form<GroupCreateForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Database unavailable".to_string()),
            )
        }
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };
    if !is_admin_user(&state, user_id) {
        return (
            StatusCode::FORBIDDEN,
            Html("Admin role required".to_string()),
        );
    }

    let name = form.name.unwrap_or_default();
    let display = form.display_name.clone().unwrap_or_else(|| name.clone());
    if let Err(reason) = validate_group_name(&name) {
        return (StatusCode::BAD_REQUEST, Html(reason.to_string()));
    }

    let _ = diesel::sql_query(
        "INSERT INTO rbac_groups (id, name, display_name, description, is_active, created_by, created_at, updated_at)
         VALUES ($1, $2, $3, $4, true, $5, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&display)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(form.description)
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .execute(&mut conn);

    (
        StatusCode::OK,
        Html(format!(
            "Group <strong>{}</strong> created",
            html_escape(&display)
        )),
    )
}

#[cfg(test)]
mod name_validation_tests {
    use super::validate_group_name;

    #[test]
    fn rejects_ambiguous_admin_substrings() {
        for name in [
            "not-admin", "admin-readonly", "admins-in-training", "notadmin", "ADMINtrainee",
        ] {
            assert!(
                validate_group_name(name).is_err(),
                "'{name}' must be refused as an ambiguous admin name"
            );
        }
    }

    #[test]
    fn accepts_canonical_admin_and_ordinary_names() {
        for name in ["admin", "SuperAdmin", "users", "support team"] {
            assert!(
                validate_group_name(name).is_ok(),
                "'{name}' should be accepted"
            );
        }
        assert!(validate_group_name("   ").is_err());
    }
}
