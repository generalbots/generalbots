use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SipConfig {
    pub provider: String,
    pub username: String,
    pub server: String,
    pub port: u16,
    pub auth_user: Option<String>,
    pub auth_password: Option<String>,
}

impl SipConfig {
    pub fn new(provider: &str, username: &str, server: &str, port: u16) -> Self {
        Self {
            provider: provider.to_string(),
            username: username.to_string(),
            server: server.to_string(),
            port,
            auth_user: None,
            auth_password: None,
        }
    }

    pub fn with_auth(mut self, user: &str, pass: &str) -> Self {
        self.auth_user = Some(user.to_string());
        self.auth_password = Some(pass.to_string());
        self
    }

    pub fn sip_uri(&self) -> String {
        format!("sip:{}@{}:{}", self.username, self.server, self.port)
    }

    pub fn display_address(&self) -> String {
        format!("<sip:{}@{}>", self.username, self.server)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub number: String,
    pub is_ported: bool,
    pub original_operator: Option<String>,
    pub current_operator: Option<String>,
    pub sip_enabled: bool,
    pub sip_config: Option<SipConfig>,
}

impl PhoneNumber {
    pub fn new(number: &str) -> Self {
        Self {
            number: number.to_string(),
            is_ported: false,
            original_operator: None,
            current_operator: None,
            sip_enabled: false,
            sip_config: None,
        }
    }

    pub fn port_from(&mut self, operator: &str) {
        self.original_operator = Some(operator.to_string());
        self.current_operator = Some(operator.to_string());
    }

    pub fn complete_port(&mut self, new_operator: &str) {
        self.is_ported = true;
        self.current_operator = Some(new_operator.to_string());
    }

    pub fn enable_sip(&mut self, config: SipConfig) {
        self.sip_enabled = true;
        self.sip_config = Some(config);
    }

    pub fn disable_sip(&mut self) {
        self.sip_enabled = false;
        self.sip_config = None;
    }

    pub fn formatted(&self) -> String {
        let cleaned: String = self.number.chars().filter(|c| c.is_ascii_digit()).collect();
        if cleaned.len() == 13 && cleaned.starts_with("55") {
            format!(
                "+{} ({}) {}-{}",
                &cleaned[..2],
                &cleaned[2..4],
                &cleaned[4..9],
                &cleaned[9..]
            )
        } else if cleaned.len() >= 10 {
            format!("+{}", cleaned)
        } else {
            self.number.clone()
        }
    }
}

pub struct NumberPortabilityChecker;

impl NumberPortabilityChecker {
    pub fn check_portability(number: &str) -> Result<PortabilityStatus, String> {
        let cleaned: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
        if cleaned.len() < 10 {
            return Err("Invalid phone number".to_string());
        }

        let operator = Self::identify_operator(&cleaned);
        Ok(PortabilityStatus {
            number: cleaned,
            operator,
            can_port: true,
            porting_time_days: 3,
            estimated_cost_brl: 0.0,
        })
    }

    fn identify_operator(number: &str) -> String {
        if number.starts_with("5511") || number.starts_with("5512") || number.starts_with("5513") {
            "Vivo".to_string()
        } else if number.starts_with("5521") || number.starts_with("5522") {
            "Claro".to_string()
        } else if number.starts_with("5531") || number.starts_with("5532") {
            "TIM".to_string()
        } else if number.starts_with("5541") || number.starts_with("5542") {
            "Oi".to_string()
        } else {
            "Unknown".to_string()
        }
    }
}

pub struct PortabilityStatus {
    pub number: String,
    pub operator: String,
    pub can_port: bool,
    pub porting_time_days: u32,
    pub estimated_cost_brl: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sip_uri() {
        let sip = SipConfig::new("provider", "user", "sip.ex.com", 5060);
        assert_eq!(sip.sip_uri(), "sip:user@sip.ex.com:5060");
    }

    #[test]
    fn test_phone_formatting() {
        let pn = PhoneNumber::new("5511999998888");
        let formatted = pn.formatted();
        assert!(formatted.contains('+'));
        assert!(formatted.contains('('));
    }

    #[test]
    fn test_port_flow() {
        let mut pn = PhoneNumber::new("5511999998888");
        pn.port_from("Vivo");
        assert!(!pn.is_ported);
        assert_eq!(pn.original_operator, Some("Vivo".to_string()));
        pn.complete_port("Claro");
        assert!(pn.is_ported);
        assert_eq!(pn.current_operator, Some("Claro".to_string()));
    }

    #[test]
    fn test_sip_enable_disable() {
        let mut pn = PhoneNumber::new("5511999998888");
        let sip = SipConfig::new("Twilio", "user", "sip.twilio.com", 5060);
        pn.enable_sip(sip);
        assert!(pn.sip_enabled);
        assert!(pn.sip_config.is_some());
        pn.disable_sip();
        assert!(!pn.sip_enabled);
        assert!(pn.sip_config.is_none());
    }

    #[test]
    fn test_check_portability() {
        let result = NumberPortabilityChecker::check_portability("5511999998888");
        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(status.can_port);
    }
}
