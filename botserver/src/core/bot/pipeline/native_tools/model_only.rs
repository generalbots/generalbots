//! A tool behind a permission card shows its outcome on its step; the texts
//! it writes while it runs are its result for the model, which answers the
//! person in their language. While a [`ModelOnly`] scope lives for a session,
//! `send` records the text for the model without streaming it, and `persist`
//! keeps it out of the conversation history.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use uuid::Uuid;

static SCOPES: OnceLock<Mutex<HashMap<Uuid, usize>>> = OnceLock::new();

fn scopes() -> MutexGuard<'static, HashMap<Uuid, usize>> {
    let map = SCOPES.get_or_init(|| Mutex::new(HashMap::new()));
    match map.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Keeps the session's tool texts for the model only until dropped.
#[cfg(feature = "agent-vm")]
pub struct ModelOnly(Uuid);

#[cfg(feature = "agent-vm")]
impl ModelOnly {
    #[must_use]
    pub fn begin(session_id: Uuid) -> Self {
        *scopes().entry(session_id).or_insert(0) += 1;
        Self(session_id)
    }
}

#[cfg(feature = "agent-vm")]
impl Drop for ModelOnly {
    fn drop(&mut self) {
        let mut map = scopes();
        if let Some(count) = map.get_mut(&self.0) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                map.remove(&self.0);
            }
        }
    }
}

/// Whether the session's tool texts currently go to the model only.
#[must_use]
pub fn model_only(session_id: Uuid) -> bool {
    scopes().get(&session_id).is_some_and(|count| *count > 0)
}

#[cfg(all(test, feature = "agent-vm"))]
mod tests {
    use super::*;

    #[test]
    fn a_scope_hides_only_its_session_and_ends_when_dropped() {
        let session = Uuid::new_v4();
        let other = Uuid::new_v4();
        assert!(!model_only(session));
        {
            let outer = ModelOnly::begin(session);
            let inner = ModelOnly::begin(session);
            assert!(model_only(session));
            assert!(!model_only(other));
            drop(inner);
            assert!(model_only(session), "the outer scope still holds");
            drop(outer);
        }
        assert!(!model_only(session));
    }
}
