//! Whether the person's workspace is already known to run, so a warm turn
//! does not wait again for the Incus round trip (about 2 s through WSL) of a
//! workspace check that just succeeded.
//!
//! Every successful check marks the person verified for [`FRESH_FOR`].
//! Within that time a turn trusts the records (status `running` and the
//! conversation folder row) and the check itself runs again beside the turn
//! (`workspace_start::check_in_background`). After a restart of the server
//! nothing is verified, so the first turn checks for real.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use uuid::Uuid;

/// How long a successful workspace check is trusted.
pub const FRESH_FOR: Duration = Duration::from_secs(10 * 60);
const MAX_USERS: usize = 4096;

fn verified() -> MutexGuard<'static, HashMap<Uuid, Instant>> {
    static VERIFIED: OnceLock<Mutex<HashMap<Uuid, Instant>>> = OnceLock::new();
    VERIFIED.get_or_init(Mutex::default).lock().unwrap_or_else(PoisonError::into_inner)
}

/// Records the result of a workspace check for `user_id`.
pub fn note_check(user_id: Uuid, running: bool) {
    let mut map = verified();
    if !running {
        map.remove(&user_id);
        return;
    }
    if map.len() >= MAX_USERS && !map.contains_key(&user_id) {
        map.retain(|_, at| at.elapsed() < FRESH_FOR);
        if map.len() >= MAX_USERS {
            map.clear();
        }
    }
    map.insert(user_id, Instant::now());
}

/// True when a check for `user_id` succeeded within [`FRESH_FOR`].
pub fn verified_recently(user_id: Uuid) -> bool {
    verified().get(&user_id).is_some_and(|at| at.elapsed() < FRESH_FOR)
}

/// The container of a workspace verified recently that the records still show
/// running. `None` means "check". This conversation's folder is made beside
/// the turn: no engine works in it and a copied attachment makes its own
/// parent folder, so a new conversation's first turn does not wait for it.
pub async fn ready_vm(session_id: Uuid, user_id: Uuid) -> Option<String> {
    if user_id.is_nil()
        || !verified_recently(user_id)
        || !crate::core::bot::pipeline::agent_turn::is_enabled(session_id)
    {
        return None;
    }
    let service = crate::core::bot::agent_vm_hook::service()?;
    let row = botagent::vm::get_user_vm(service, &user_id).ok()??;
    if row.status != botagent::vm::STATUS_RUNNING {
        return None;
    }
    let vm_name = row.vm_name.clone();
    tokio::spawn(async move {
        if let Err((_, error)) = botagent::vm::ensure_conversation_folder(service, &row, &session_id.to_string()).await {
            botlog::safe_warn!("conversation folder for {session_id}: {error}");
        }
    });
    Some(vm_name)
}

/// The workspace container for this turn: [`ready_vm`] when it is known to
/// run, otherwise a full check (start or provisioning included).
pub async fn workspace_vm(session_id: Uuid, user_id: Uuid, bot_uuid: Uuid) -> Option<String> {
    if let Some(vm_name) = ready_vm(session_id, user_id).await {
        return Some(vm_name);
    }
    let row = crate::core::bot::agent_vm_hook::ensure_agent_vm(session_id, user_id, bot_uuid).await;
    note_check(user_id, row.is_some());
    row.map(|row| row.vm_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_successful_check_is_trusted_and_a_failed_one_forgets_it() {
        let user = Uuid::new_v4();
        assert!(!verified_recently(user));
        note_check(user, true);
        assert!(verified_recently(user));
        note_check(user, false);
        assert!(!verified_recently(user));
    }

    #[tokio::test]
    async fn a_guest_or_an_unverified_person_is_never_trusted() {
        assert_eq!(ready_vm(Uuid::new_v4(), Uuid::nil()).await, None);
        assert_eq!(ready_vm(Uuid::new_v4(), Uuid::new_v4()).await, None);
    }
}
