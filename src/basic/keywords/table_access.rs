/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the AGPL-3.0.                                                |
|                                                                             |
| According to our dual licensing model, this program can be used either      |
| under the terms of the GNU Affero General Public License, version 3,        |
| or under a proprietary license.                                             |
|                                                                             |
| The texts of the GNU Affero General Public License with an additional       |
| permission and of our proprietary license can be found at and               |
| in the LICENSE file you have received along with this program.              |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the                |
| GNU Affero General Public License for more details.                         |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the AGPLv3 does not imply a              |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

use crate::core::shared::models::UserSession;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use log::{trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserRoles {
    pub roles: Vec<String>,
    pub user_id: Option<uuid::Uuid>,
}

impl UserRoles {
    pub fn new(roles: Vec<String>) -> Self {
        Self {
            roles: roles.into_iter().map(|r| r.to_lowercase()).collect(),
            user_id: None,
        }
    }

    pub fn with_user_id(roles: Vec<String>, user_id: uuid::Uuid) -> Self {
        Self {
            roles: roles.into_iter().map(|r| r.to_lowercase()).collect(),
            user_id: Some(user_id),
        }
    }

    pub fn anonymous() -> Self {
        Self::default()
    }

    pub fn from_user_session(session: &UserSession) -> Self {
        let mut roles = Vec::new();

        // Try different keys where roles might be stored
        let role_keys = ["roles", "user_roles", "zitadel_roles"];

        for key in role_keys {
            if let Some(value) = session.context_data.get(key) {
                match value {
                    // Array of strings
                    Value::Array(arr) => {
                        for item in arr {
                            if let Value::String(s) = item {
                                roles.push(s.trim().to_lowercase());
                            }
                        }
                        if !roles.is_empty() {
                            break;
                        }
                    }
                    // Semicolon-separated string
                    Value::String(s) => {
                        roles = s
                            .split(';')
                            .map(|r| r.trim().to_lowercase())
                            .filter(|r| !r.is_empty())
                            .collect();
                        if !roles.is_empty() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Also check if user is marked as admin in context
        if matches!(session.context_data.get("is_admin"), Some(Value::Bool(true)))
            && !roles.contains(&"admin".to_string())
        {
            roles.push("admin".to_string());
        }

        Self {
            roles,
            user_id: Some(session.user_id),
        }
    }

    pub fn has_access(&self, required_roles: &[String]) -> bool {
        if required_roles.is_empty() {
            return true; // No roles specified = everyone has access
        }

        // Check if user has any of the required roles
        self.roles.iter().any(|user_role| {
            required_roles
                .iter()
                .any(|req| req.to_lowercase() == *user_role)
        })
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == &role.to_lowercase())
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableAccessInfo {
    pub table_name: String,
    pub read_roles: Vec<String>,
    pub write_roles: Vec<String>,
    pub field_read_roles: HashMap<String, Vec<String>>,
    pub field_write_roles: HashMap<String, Vec<String>>,
}

impl TableAccessInfo {
    pub fn can_read(&self, user_roles: &UserRoles) -> bool {
        user_roles.has_access(&self.read_roles)
    }

    pub fn can_write(&self, user_roles: &UserRoles) -> bool {
        user_roles.has_access(&self.write_roles)
    }

    pub fn can_read_field(&self, field_name: &str, user_roles: &UserRoles) -> bool {
        if let Some(field_roles) = self.field_read_roles.get(field_name) {
            user_roles.has_access(field_roles)
        } else {
            true // No field-level restriction
        }
    }

    pub fn can_write_field(&self, field_name: &str, user_roles: &UserRoles) -> bool {
        if let Some(field_roles) = self.field_write_roles.get(field_name) {
            user_roles.has_access(field_roles)
        } else {
            true // No field-level restriction
        }
    }

    pub fn get_restricted_read_fields(&self, user_roles: &UserRoles) -> Vec<String> {
        self.field_read_roles
            .iter()
            .filter(|(_, roles)| !user_roles.has_access(roles))
            .map(|(field, _)| field.clone())
            .collect()
    }

    pub fn get_restricted_write_fields(&self, user_roles: &UserRoles) -> Vec<String> {
        self.field_write_roles
            .iter()
            .filter(|(_, roles)| !user_roles.has_access(roles))
            .map(|(field, _)| field.clone())
            .collect()
    }
}

#[derive(QueryableByName, Debug)]
struct TableDefRow {
    #[diesel(sql_type = Text)]
    table_name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    read_roles: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    write_roles: Option<String>,
}

#[derive(QueryableByName, Debug)]
struct FieldDefRow {
    #[diesel(sql_type = Text)]
    field_name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    read_roles: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    write_roles: Option<String>,
}

pub fn load_table_access_info(
    conn: &mut diesel::PgConnection,
    table_name: &str,
) -> Option<TableAccessInfo> {
    // Query table-level permissions
    let table_result: Result<TableDefRow, _> = sql_query(
        "SELECT table_name, read_roles, write_roles
         FROM dynamic_table_definitions
         WHERE table_name = $1
         LIMIT 1",
    )
    .bind::<Text, _>(table_name)
    .get_result(conn);

    let Ok(table_def) = table_result else {
        trace!(
            "No table definition found for '{table_name}', allowing open access"
        );
        return None;
    };

    let mut info = TableAccessInfo {
        table_name: table_def.table_name,
        read_roles: parse_roles_string(table_def.read_roles.as_ref()),
        write_roles: parse_roles_string(table_def.write_roles.as_ref()),
        field_read_roles: HashMap::new(),
        field_write_roles: HashMap::new(),
    };

    let fields_result: Result<Vec<FieldDefRow>, _> = sql_query(
        "SELECT f.field_name, f.read_roles, f.write_roles
         FROM dynamic_table_fields f
         JOIN dynamic_table_definitions t ON f.table_definition_id = t.id
         WHERE t.table_name = $1",
    )
    .bind::<Text, _>(table_name)
    .get_results(conn);

    if let Ok(fields) = fields_result {
        for field in fields {
            let field_read = parse_roles_string(field.read_roles.as_ref());
            let field_write = parse_roles_string(field.write_roles.as_ref());

            if !field_read.is_empty() {
                info.field_read_roles
                    .insert(field.field_name.clone(), field_read);
            }
            if !field_write.is_empty() {
                info.field_write_roles.insert(field.field_name, field_write);
            }
        }
    }

    trace!(
        "Loaded access info for table '{}': read_roles={:?}, write_roles={:?}, field_restrictions={}",
        info.table_name,
        info.read_roles,
        info.write_roles,
        info.field_read_roles.len() + info.field_write_roles.len()
    );

    Some(info)
}

fn parse_roles_string(roles: Option<&String>) -> Vec<String> {
    roles
        .map(|s| {
            s.split(';')
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub fn check_table_access(
    conn: &mut diesel::PgConnection,
    table_name: &str,
    user_roles: &UserRoles,
    access_type: AccessType,
) -> Result<Option<TableAccessInfo>, String> {
    let access_info = load_table_access_info(conn, table_name);

    if let Some(ref info) = access_info {
        let has_access = match access_type {
            AccessType::Read => info.can_read(user_roles),
            AccessType::Write => info.can_write(user_roles),
        };

        if !has_access {
            let action = match access_type {
                AccessType::Read => "read from",
                AccessType::Write => "write to",
            };
            warn!(
                "Access denied: user {:?} cannot {} table '{}'",
                user_roles.user_id, action, table_name
            );
            return Err(format!(
                "Access denied: insufficient permissions to {} table '{}'",
                action, table_name
            ));
        }
    }

    Ok(access_info)
}

pub fn check_field_write_access(
    fields: &[String],
    user_roles: &UserRoles,
    access_info: &Option<TableAccessInfo>,
) -> Result<(), String> {
    let Some(info) = access_info else {
        return Ok(()); // No access info = allow all
    };

    let mut denied_fields = Vec::new();

    for field in fields {
        if !info.can_write_field(field, user_roles) {
            denied_fields.push(field.clone());
        }
    }

    if denied_fields.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Access denied: insufficient permissions to write field(s): {}",
            denied_fields.join(", ")
        ))
    }
}

pub fn filter_fields_by_role(
    data: Value,
    user_roles: &UserRoles,
    access_info: &Option<TableAccessInfo>,
) -> Value {
    let Some(info) = access_info else {
        return data; // No access info = return all fields
    };

    match data {
        Value::Object(mut map) => {
            let restricted = info.get_restricted_read_fields(user_roles);

            for field in restricted {
                trace!("Filtering out field '{}' due to role restriction", field);
                map.remove(&field);
            }

            Value::Object(map)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|v| filter_fields_by_role(v, user_roles, access_info))
                .collect(),
        ),
        other => other,
    }
}

pub fn filter_write_fields(
    data: Value,
    user_roles: &UserRoles,
    access_info: &Option<TableAccessInfo>,
) -> (Value, Vec<String>) {
    let Some(info) = access_info else {
        return (data, Vec::new()); // No access info = allow all
    };

    match data {
        Value::Object(mut map) => {
            let restricted = info.get_restricted_write_fields(user_roles);
            let mut removed = Vec::new();

            for field in &restricted {
                if map.contains_key(field) {
                    trace!(
                        "Removing field '{}' from write data due to role restriction",
                        field
                    );
                    map.remove(field);
                    removed.push(field.clone());
                }
            }

            (Value::Object(map), removed)
        }
        other => (other, Vec::new()),
    }
}

/// Get column names for a table from the database schema
pub fn get_table_columns(conn: &mut PgConnection, table_name: &str) -> Vec<String> {
    use diesel::prelude::*;
    use diesel::sql_types::Text;

    // Define a struct for the query result
    #[derive(diesel::QueryableByName)]
    struct ColumnName {
        #[diesel(sql_type = Text)]
        column_name: String,
    }

    // Query information_schema to get column names
    diesel::sql_query(
        "SELECT column_name FROM information_schema.columns WHERE table_name = $1 ORDER BY ordinal_position"
    )
    .bind::<Text, _>(table_name)
    .load::<ColumnName>(conn)
    .unwrap_or_default()
    .into_iter()
    .map(|c| c.column_name)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_roles_has_access() {
        let roles = UserRoles::new(vec!["admin".to_string(), "manager".to_string()]);

        // Empty roles = everyone allowed
        assert!(roles.has_access(&[]));

        // User has admin role
        assert!(roles.has_access(&["admin".to_string()]));

        // User has manager role
        assert!(roles.has_access(&["manager".to_string(), "superuser".to_string()]));

        // User doesn't have superuser role only
        assert!(!roles.has_access(&["superuser".to_string()]));
    }

    #[test]
    fn test_user_roles_case_insensitive() {
        let roles = UserRoles::new(vec!["Admin".to_string()]);

        assert!(roles.has_access(&["admin".to_string()]));
        assert!(roles.has_access(&["ADMIN".to_string()]));
        assert!(roles.has_access(&["Admin".to_string()]));
    }

    #[test]
    fn test_anonymous_user() {
        let roles = UserRoles::anonymous();

        // Anonymous can access if no roles required
        assert!(roles.has_access(&[]));

        // Anonymous cannot access if roles required
        assert!(!roles.has_access(&["admin".to_string()]));
    }

    #[test]
    fn test_table_access_info_field_restrictions() {
        let mut info = TableAccessInfo {
            table_name: "contacts".to_string(),
            read_roles: vec![],
            write_roles: vec![],
            field_read_roles: HashMap::new(),
            field_write_roles: HashMap::new(),
        };

        info.field_read_roles
            .insert("ssn".to_string(), vec!["admin".to_string()]);
        info.field_write_roles
            .insert("salary".to_string(), vec!["hr".to_string()]);

        let admin = UserRoles::new(vec!["admin".to_string()]);
        let hr = UserRoles::new(vec!["hr".to_string()]);
        let user = UserRoles::new(vec!["user".to_string()]);

        // Admin can read SSN
        assert!(info.can_read_field("ssn", &admin));
        // Regular user cannot read SSN
        assert!(!info.can_read_field("ssn", &user));

        // HR can write salary
        assert!(info.can_write_field("salary", &hr));
        // Admin cannot write salary (different role)
        assert!(!info.can_write_field("salary", &admin));

        // Everyone can read/write unrestricted fields
        assert!(info.can_read_field("name", &user));
        assert!(info.can_write_field("name", &user));
    }

    #[test]
    fn test_filter_fields_by_role() {
        let mut info = TableAccessInfo::default();
        info.field_read_roles
            .insert("secret".to_string(), vec!["admin".to_string()]);

        let data = serde_json::json!({
            "id": 1,
            "name": "John",
            "secret": "classified"
        });

        let user = UserRoles::new(vec!["user".to_string()]);
        let filtered = filter_fields_by_role(data.clone(), &user, &Some(info.clone()));

        assert!(filtered.get("id").is_some());
        assert!(filtered.get("name").is_some());
        assert!(filtered.get("secret").is_none());

        // Admin can see everything
        let admin = UserRoles::new(vec!["admin".to_string()]);
        let not_filtered = filter_fields_by_role(data, &admin, &Some(info));

        assert!(not_filtered.get("secret").is_some());
    }

    #[test]
    fn test_parse_roles_string() {
        assert_eq!(parse_roles_string(None), Vec::<String>::new());
        assert_eq!(
            parse_roles_string(Some("".to_string()).as_ref()),
            Vec::<String>::new()
        );
        assert_eq!(
            parse_roles_string(Some("admin".to_string()).as_ref()),
            vec!["admin"]
        );
        assert_eq!(
            parse_roles_string(Some("admin;manager".to_string()).as_ref()),
            vec!["admin", "manager"]
        );
        assert_eq!(
            parse_roles_string(Some(" admin ; manager ; hr ".to_string()).as_ref()),
            vec!["admin", "manager", "hr"]
        );
    }
}
