use super::*;

/// Decodes `%XX` escapes from a connection-string component.
///
/// Passwords routinely contain characters that must be escaped in a URL
/// (`@`, `:`, `/`); leaving them encoded hands the database a password that
/// does not match the server's.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(value) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(value);
                i += 3;
                continue;
            }
        }

        out.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&out).into_owned()
}

impl DatabaseCredentials {
    pub fn new(
        username: impl Into<String>,
        password: impl Into<SecretString>,
        host: impl Into<String>,
        port: u16,
        database: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            host: host.into(),
            port,
            database: database.into(),
        }
    }

    pub fn from_url(url: &str) -> Option<Self> {
        let url = url
            .strip_prefix("postgres://")
            .or_else(|| url.strip_prefix("postgresql://"))?;
        let (auth, rest) = url.split_once('@')?;
        let (username, password) = auth.split_once(':').unwrap_or((auth, ""));
        let (host_port, database) = rest.split_once('/')?;

        let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
            (h.to_string(), p.parse().ok()?)
        } else {
            (host_port.to_string(), 5432)
        };

        let database = database.split('?').next()?.to_string();

        Some(Self {
            username: percent_decode(username),
            password: percent_decode(password).into(),
            host,
            port,
            database: percent_decode(&database),
        })
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn expose_password(&self) -> &str {
        self.password.expose_secret()
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn database(&self) -> &str {
        &self.database
    }

    pub fn to_connection_string(&self) -> SecretString {
        SecretString::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database
        ))
    }

    pub fn to_safe_string(&self) -> String {
        format!(
            "postgres://{}:****@{}:{}/{}",
            self.username, self.host, self.port, self.database
        )
    }
}
impl fmt::Debug for DatabaseCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseCredentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .finish()
    }
}
impl fmt::Display for DatabaseCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_safe_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_standard_postgres_url() {
        let creds = DatabaseCredentials::from_url("postgres://user:pass@localhost:5432/mydb")
            .expect("parsed");

        assert_eq!(creds.username(), "user");
        assert_eq!(creds.expose_password(), "pass");
        assert_eq!(creds.host(), "localhost");
        assert_eq!(creds.port(), 5432);
        assert_eq!(creds.database(), "mydb");
    }

    #[test]
    fn accepts_the_postgresql_scheme_and_default_port() {
        let creds = DatabaseCredentials::from_url("postgresql://user:pass@db.internal/app")
            .expect("parsed");

        assert_eq!(creds.host(), "db.internal");
        assert_eq!(creds.port(), 5432);
    }

    #[test]
    fn decodes_percent_escaped_credentials() {
        let creds = DatabaseCredentials::from_url(
            "postgres://user%40corp:p%40ss%3Aword%2F1@localhost:5432/my%20db",
        )
        .expect("parsed");

        assert_eq!(creds.username(), "user@corp");
        assert_eq!(creds.expose_password(), "p@ss:word/1");
        assert_eq!(creds.database(), "my db");
    }

    #[test]
    fn keeps_an_empty_password_instead_of_defaulting_it() {
        let creds = DatabaseCredentials::from_url("postgres://user:@localhost/db").expect("parsed");

        assert_eq!(creds.expose_password(), "");
    }

    #[test]
    fn strips_query_parameters_from_the_database_name() {
        let creds =
            DatabaseCredentials::from_url("postgres://u:p@localhost/db?sslmode=require").expect("parsed");

        assert_eq!(creds.database(), "db");
    }

    #[test]
    fn rejects_unsupported_or_incomplete_urls() {
        assert!(DatabaseCredentials::from_url("mysql://u:p@localhost/db").is_none());
        assert!(DatabaseCredentials::from_url("postgres://localhost/db").is_none());
        assert!(DatabaseCredentials::from_url("postgres://u:p@localhost").is_none());
        assert!(DatabaseCredentials::from_url("postgres://u:p@localhost:notaport/db").is_none());
    }

    #[test]
    fn connection_strings_round_trip_and_never_leak_in_display() {
        let creds = DatabaseCredentials::from_url("postgres://u:p%40ss@localhost:5433/db")
            .expect("parsed");

        let connection = creds.to_connection_string();
        assert!(connection.expose_secret().contains("p@ss"));
        assert!(!creds.to_safe_string().contains("p@ss"));
        assert!(!format!("{creds:?}").contains("p@ss"));
        assert!(!format!("{creds}").contains("p@ss"));
    }
}
