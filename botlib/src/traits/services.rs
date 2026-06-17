use crate::traits::{
    BoxFutureBool, BoxFutureOptionValue, BoxFutureString, BoxFutureUnit, BoxFutureValue,
    BoxFutureVecValue,
};
use std::fmt::Debug;

#[cfg(feature = "database")]
pub trait BotDatabaseService: Send + Sync + Debug {
    fn get_bot_pool(
        &self,
        bot_id: uuid::Uuid,
    ) -> Option<crate::db_pool::DbPool>;

    fn create_table_in_bot_database(
        &self,
        bot_id: uuid::Uuid,
        sql: &str,
    ) -> Result<(), String>;

    fn sync_all_bot_databases(&self) -> Result<(), String>;
}

pub trait JwtService: Send + Sync + Debug {
    fn validate_access_token(
        &self,
        token: &str,
    ) -> Result<serde_json::Value, String>;

    fn generate_access_token(
        &self,
        user_id: uuid::Uuid,
        claims: serde_json::Value,
    ) -> Result<String, String>;
}

pub trait RbacService: Send + Sync + Debug {
    fn check_permission(
        &self,
        user_id: uuid::Uuid,
        resource: &str,
        action: &str,
    ) -> BoxFutureBool;

    fn register_routes(
        &self,
        default_permissions: serde_json::Value,
    ) -> BoxFutureUnit;
}

pub trait AuthServiceTrait: Send + Sync + Debug {
    fn api_url(&self) -> String;

    fn client_id(&self) -> String;

    fn client_secret(&self) -> String;

    fn get_access_token(
        &self,
    ) -> BoxFutureString;

    fn get_user_by_token(
        &self,
        token: &str,
    ) -> BoxFutureOptionValue;

    fn list_users(
        &self,
        limit: i64,
        offset: i64,
    ) -> BoxFutureValue {
        let _ = (limit, offset);
        Box::pin(async { Err("list_users: not implemented".to_string()) })
    }

    fn create_user(
        &self,
        email: &str,
        first_name: &str,
        last_name: &str,
        username: Option<&str>,
    ) -> BoxFutureString {
        let _ = (email, first_name, last_name, username);
        Box::pin(async { Err("create_user: not implemented".to_string()) })
    }

    fn add_org_member(
        &self,
        org_id: &str,
        user_id: &str,
        roles: Vec<String>,
    ) -> BoxFutureUnit {
        let _ = (org_id, user_id, roles);
        Box::pin(async { Err("add_org_member: not implemented".to_string()) })
    }

    fn set_user_password(
        &self,
        user_id: &str,
        password: &str,
    ) -> BoxFutureUnit {
        let _ = (user_id, password);
        Box::pin(async { Err("set_user_password: not implemented".to_string()) })
    }

    fn list_organizations(
        &self,
    ) -> BoxFutureValue {
        Box::pin(async { Err("list_organizations: not implemented".to_string()) })
    }

    fn get_organization(
        &self,
        org_id: &str,
    ) -> BoxFutureValue {
        let _ = org_id;
        Box::pin(async { Err("get_organization: not implemented".to_string()) })
    }

    fn create_organization(
        &self,
        name: &str,
    ) -> BoxFutureString {
        let _ = name;
        Box::pin(async { Err("create_organization: not implemented".to_string()) })
    }

    fn http_get(
        &self,
        url: String,
    ) -> BoxFutureValue {
        let _ = url;
        Box::pin(async { Err("http_get: not implemented".to_string()) })
    }

    fn http_post(
        &self,
        url: String,
        body: serde_json::Value,
    ) -> BoxFutureValue {
        let _ = (url, body);
        Box::pin(async { Err("http_post: not implemented".to_string()) })
    }

    fn get_user(
        &self,
        user_id: &str,
    ) -> BoxFutureValue {
        let _ = user_id;
        Box::pin(async { Err("get_user: not implemented".to_string()) })
    }

    fn search_users(
        &self,
        query: &str,
    ) -> BoxFutureVecValue {
        let _ = query;
        Box::pin(async { Err("search_users: not implemented".to_string()) })
    }

    fn search_users_by_phone(
        &self,
        phone: &str,
    ) -> BoxFutureVecValue {
        let _ = phone;
        Box::pin(async { Err("search_users_by_phone: not implemented".to_string()) })
    }

    fn search_users_by_email(
        &self,
        email: &str,
    ) -> BoxFutureVecValue {
        let _ = email;
        Box::pin(async { Err("search_users_by_email: not implemented".to_string()) })
    }

    fn search_users_by_metadata(
        &self,
        key: &str,
        value: &str,
    ) -> BoxFutureVecValue {
        let _ = (key, value);
        Box::pin(async { Err("search_users_by_metadata: not implemented".to_string()) })
    }

    fn find_or_create_user_by_phone(
        &self,
        phone: &str,
        first_name: &str,
        last_name: &str,
    ) -> BoxFutureString {
        let _ = (phone, first_name, last_name);
        Box::pin(async { Err("find_or_create_user_by_phone: not implemented".to_string()) })
    }

    fn get_user_memberships(
        &self,
        user_id: &str,
        offset: i64,
        limit: i64,
    ) -> BoxFutureValue {
        let _ = (user_id, offset, limit);
        Box::pin(async { Err("get_user_memberships: not implemented".to_string()) })
    }

    fn remove_org_member(
        &self,
        org_id: &str,
        user_id: &str,
    ) -> BoxFutureUnit {
        let _ = (org_id, user_id);
        Box::pin(async { Err("remove_org_member: not implemented".to_string()) })
    }

    fn get_org_members(
        &self,
        org_id: &str,
    ) -> BoxFutureVecValue {
        let _ = org_id;
        Box::pin(async { Err("get_org_members: not implemented".to_string()) })
    }

    fn http_patch(
        &self,
        url: String,
        body: serde_json::Value,
    ) -> BoxFutureValue {
        let _ = (url, body);
        Box::pin(async { Err("http_patch: not implemented".to_string()) })
    }

    fn http_delete(
        &self,
        url: String,
    ) -> BoxFutureValue {
        let _ = url;
        Box::pin(async { Err("http_delete: not implemented".to_string()) })
    }

    fn http_put(
        &self,
        url: String,
        body: serde_json::Value,
    ) -> BoxFutureValue {
        let _ = (url, body);
        Box::pin(async { Err("http_put: not implemented".to_string()) })
    }
}

pub trait TaskSchedulerService: Send + Sync + Debug {
    fn schedule_task(
        &self,
        task_id: &str,
        cron_expr: &str,
    ) -> BoxFutureUnit;
}

pub trait TaskEngineService: Send + Sync + Debug {
    fn execute_task(
        &self,
        task_id: &str,
    ) -> BoxFutureUnit;
}

pub trait MetricsService: Send + Sync + Debug {
    fn record_metric(
        &self,
        name: &str,
        value: f64,
    );
}
