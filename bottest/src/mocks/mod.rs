
mod llm;
mod teams;
mod whatsapp;
mod zitadel;

pub use llm::MockLLM;
pub use teams::MockTeams;
pub use whatsapp::MockWhatsApp;
pub use zitadel::MockZitadel;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockRegistry {
    pub llm: Option<MockLLM>,
    pub whatsapp: Option<MockWhatsApp>,
    pub teams: Option<MockTeams>,
    pub zitadel: Option<MockZitadel>,
}

impl MockRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            llm: None,
            whatsapp: None,
            teams: None,
            zitadel: None,
        }
    }

    #[must_use]
    pub const fn llm(&self) -> &MockLLM {
        self.llm.as_ref().expect("LLM mock not configured")
    }

    #[must_use]
    pub const fn whatsapp(&self) -> &MockWhatsApp {
        self.whatsapp.as_ref().expect("WhatsApp mock not configured")
    }

    #[must_use]
    pub const fn teams(&self) -> &MockTeams {
        self.teams.as_ref().expect("Teams mock not configured")
    }

    #[must_use]
    pub const fn zitadel(&self) -> &MockZitadel {
        self.zitadel.as_ref().expect("Zitadel mock not configured")
    }

    pub fn verify_all(&self) -> Result<()> {
        if let Some(ref llm) = self.llm {
            llm.verify()?;
        }
        if let Some(ref whatsapp) = self.whatsapp {
            whatsapp.verify()?;
        }
        if let Some(ref teams) = self.teams {
            teams.verify()?;
        }
        if let Some(ref zitadel) = self.zitadel {
            zitadel.verify()?;
        }
        Ok(())
    }

    pub async fn reset_all(&self) {
        if let Some(ref llm) = self.llm {
            llm.reset().await;
        }
        if let Some(ref whatsapp) = self.whatsapp {
            whatsapp.reset().await;
        }
        if let Some(ref teams) = self.teams {
            teams.reset().await;
        }
        if let Some(ref zitadel) = self.zitadel {
            zitadel.reset().await;
        }
    }
}

impl Default for MockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Expectation {
    pub name: String,
    pub expected_calls: Option<usize>,
    pub actual_calls: usize,
    pub matched: bool,
}

impl Expectation {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            expected_calls: None,
            actual_calls: 0,
            matched: false,
        }
    }

    #[must_use]
    pub const fn times(mut self, n: usize) -> Self {
        self.expected_calls = Some(n);
        self
    }

    pub const fn record_call(&mut self) {
        self.actual_calls += 1;
        self.matched = true;
    }

    pub fn verify(&self) -> Result<()> {
        if let Some(expected) = self.expected_calls {
            if self.actual_calls != expected {
                anyhow::bail!(
                    "Expectation '{}' expected {} calls but got {}",
                    self.name,
                    expected,
                    self.actual_calls
                );
            }
        }
        Ok(())
    }
}

pub type ExpectationStore = Arc<Mutex<HashMap<String, Expectation>>>;

#[must_use]
pub fn new_expectation_store() -> ExpectationStore {
    Arc::new(Mutex::new(HashMap::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expectation_basic() {
        let mut exp = Expectation::new("test");
        assert_eq!(exp.actual_calls, 0);
        assert!(!exp.matched);

        exp.record_call();
        assert_eq!(exp.actual_calls, 1);
        assert!(exp.matched);
    }

    #[test]
    fn test_expectation_times() {
        let mut exp = Expectation::new("test").times(2);
        exp.record_call();
        exp.record_call();

        assert!(exp.verify().is_ok());
    }

    #[test]
    fn test_expectation_times_fail() {
        let mut exp = Expectation::new("test").times(2);
        exp.record_call();

        assert!(exp.verify().is_err());
    }

    #[test]
    fn test_mock_registry_default() {
        let registry = MockRegistry::new();
        assert!(registry.llm.is_none());
        assert!(registry.whatsapp.is_none());
        assert!(registry.teams.is_none());
        assert!(registry.zitadel.is_none());
    }
}
