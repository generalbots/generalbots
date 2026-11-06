use super::Permission;

impl Permission {
    pub fn from_alias(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        Self::from_alias_mail_admin(&s_lower)
            .or_else(|| Self::from_alias_chat_business(&s_lower))
            .or_else(|| Self::from_alias_integrations(&s_lower))
            .or_else(|| Self::from_alias_workspace(&s_lower))
            .or_else(|| Self::from_alias_security_more(&s_lower))
            .or_else(|| Self::from_alias_tickets_more(&s_lower))
            .or_else(|| Self::from_alias_canvas(&s_lower))
            .or_else(|| Self::from_alias_code(&s_lower))
    }
}

