pub enum FraudAction {
    Allow,
    Flag,
    Review,
    Block,
}

impl FraudAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Flag => "flag",
            Self::Review => "review",
            Self::Block => "block",
        }
    }
}

#[cfg(test)]
#[path = "actions_tests.rs"]
mod actions_tests;
