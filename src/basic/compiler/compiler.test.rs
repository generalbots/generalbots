#[cfg(test)]
mod tests {
    use super::*;
    use diesel::Connection;
    use std::sync::Mutex;
    #[cfg(test)]
    mod test_utils {
        use super::*;
        use diesel::connection::{Connection, SimpleConnection};
        use diesel::pg::Pg;
        use diesel::query_builder::QueryFragment;
        use diesel::query_builder::QueryId;
        use diesel::result::QueryResult;
        use diesel::sql_types::Untyped;
        use diesel::deserialize::Queryable;
        use std::sync::{Arc, Mutex};
        struct MockPgConnection;
        impl Connection for MockPgConnection {
            type Backend = Pg;
            type TransactionManager = diesel::connection::AnsiTransactionManager;
            fn establish(_: &str) -> diesel::ConnectionResult<Self> {
                Ok(MockPgConnection {
                    transaction_manager: diesel::connection::AnsiTransactionManager::default()
                })
            }
            fn execute(&self, _: &str) -> QueryResult<usize> {
                Ok(0)
            }
            fn load<T>(&self, _: &diesel::query_builder::SqlQuery) -> QueryResult<T>
            where
                T: Queryable<Untyped, Pg>,
            {
                unimplemented!()
            }
            fn execute_returning_count<T>(&self, _: &T) -> QueryResult<usize>
            where
                T: QueryFragment<Pg> + QueryId,
            {
                Ok(0)
            }
            fn transaction_state(&self) -> &diesel::connection::AnsiTransactionManager {
                &self.transaction_manager
            }
            fn instrumentation(&self) -> &dyn diesel::connection::Instrumentation {
                &diesel::connection::NoopInstrumentation
            }
            fn set_instrumentation(&mut self, _: Box<dyn diesel::connection::Instrumentation>) {}
            fn set_prepared_statement_cache_size(&mut self, _: usize) {}
        }
        impl AppState {
            pub fn test_default() -> Self {
                let mut state = Self::default();
                state.conn = Arc::new(Mutex::new(MockPgConnection));
                state
            }
        }
    }
    #[test]
    fn test_normalize_type() {
        let state = AppState::test_default();
        let compiler = BasicCompiler::new(Arc::new(state), uuid::Uuid::nil());
        assert_eq!(compiler.normalize_type("string"), "string");
        assert_eq!(compiler.normalize_type("integer"), "integer");
        assert_eq!(compiler.normalize_type("int"), "integer");
        assert_eq!(compiler.normalize_type("boolean"), "boolean");
        assert_eq!(compiler.normalize_type("date"), "string");
    }
    #[test]
    fn test_parse_param_line() {
        let state = AppState::test_default();
        let compiler = BasicCompiler::new(Arc::new(state), uuid::Uuid::nil());
        let line = r#"PARAM name AS string LIKE "John Doe" DESCRIPTION "User's full name""#;
        let result = compiler.parse_param_line(line).unwrap();
        assert!(result.is_some());
        let param = result.unwrap();
        assert_eq!(param.name, "name");
        assert_eq!(param.param_type, "string");
        assert_eq!(param.example, Some("John Doe".to_string()));
        assert_eq!(param.description, "User's full name");
    }
}
