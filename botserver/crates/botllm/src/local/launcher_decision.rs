/// Pure decision logic for the legacy llama-server launcher.
///
/// The function takes six booleans that describe the boot environment and
/// returns which actions the launcher should take.  It has no I/O and no
/// feature gates, so every edition compiles and tests the same code path.

/// Inputs the launcher gathers before deciding what to start.
#[derive(Debug, Clone)]
pub struct LauncherInput {
    /// Another component (runtime control) owns the chat engine on 8081.
    pub chat_managed_elsewhere: bool,
    /// The configured `llm-url` points at an external HTTPS endpoint.
    pub llm_url_is_https: bool,
    /// Something is already accepting connections on the chat port.
    pub chat_port_answering: bool,
    /// The product configuration wants a local embedding service.
    pub embedding_wanted: bool,
    /// The configured `embedding-url` points at an external HTTPS endpoint.
    pub embedding_url_is_https: bool,
    /// Something is already accepting connections on the embedding port.
    pub embedding_port_answering: bool,
}

/// What the launcher should do after the decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherDecision {
    /// Spawn `start_llm_server` for the chat model on 8081.
    pub start_chat: bool,
    /// Spawn `start_embedding_server` on 8082.
    pub start_embedding: bool,
    /// Kill existing stack llama-server processes before (re)starting.
    pub kill_allowed: bool,
    /// Call `set_embedding_server_ready(true)` immediately.
    pub mark_embedding_ready: bool,
}

/// Decide what the legacy launcher should do at boot.
///
/// The function is deterministic and free of side effects.  The launcher
/// calls it once, after probing the ports and reading the config flags,
/// and then executes exactly the returned actions.
pub fn decide(i: &LauncherInput) -> LauncherDecision {
    let chat_already = i.llm_url_is_https || i.chat_port_answering;
    let emb_already = i.embedding_url_is_https || i.embedding_port_answering;

    let want_chat = !i.chat_managed_elsewhere && !chat_already;
    let want_emb = i.embedding_wanted && !emb_already;

    // Kill is only allowed when the launcher owns the chat engine AND at
    // least one server needs (re)starting.  When chat is managed elsewhere
    // the kill would destroy the runtime-control engine.  When embedding is
    // the only thing missing but chat is managed, the embedding port is
    // free anyway, so a kill is unnecessary.
    let kill_allowed = !i.chat_managed_elsewhere && (want_chat || want_emb);

    // The ready flag is set only when embedding is confirmed running right
    // now AND wanted.  After a kill-and-restart cycle the confirmation
    // comes later (from the background health check), not from the launcher.
    let mark_ready = i.embedding_wanted && emb_already && !kill_allowed;

    LauncherDecision {
        start_chat: want_chat,
        start_embedding: want_emb,
        kill_allowed,
        mark_embedding_ready: mark_ready,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helper ──────────────────────────────────────────────────────────

    fn input(overrides: impl FnOnce(&mut LauncherInput)) -> LauncherInput {
        let mut i = LauncherInput {
            chat_managed_elsewhere: false,
            llm_url_is_https: false,
            chat_port_answering: false,
            embedding_wanted: true,
            embedding_url_is_https: false,
            embedding_port_answering: false,
        };
        overrides(&mut i);
        i
    }

    // ── current-edition parity (chat NOT managed elsewhere) ────────────

    #[test]
    fn both_ports_running_marks_ready_no_restart() {
        let d = decide(&input(|i| {
            i.chat_port_answering = true;
            i.embedding_port_answering = true;
        }));
        assert!(!d.start_chat);
        assert!(!d.start_embedding);
        assert!(!d.kill_allowed);
        assert!(d.mark_embedding_ready);
    }

    #[test]
    fn both_https_marks_ready_no_restart() {
        let d = decide(&input(|i| {
            i.llm_url_is_https = true;
            i.embedding_url_is_https = true;
        }));
        assert!(!d.start_chat);
        assert!(!d.start_embedding);
        assert!(!d.kill_allowed);
        assert!(d.mark_embedding_ready);
    }

    #[test]
    fn chat_https_embedding_running_marks_ready() {
        let d = decide(&input(|i| {
            i.llm_url_is_https = true;
            i.embedding_port_answering = true;
        }));
        assert!(!d.start_chat);
        assert!(!d.start_embedding);
        assert!(!d.kill_allowed);
        assert!(d.mark_embedding_ready);
    }

    #[test]
    fn chat_running_embedding_down_kills_and_starts_embedding() {
        let d = decide(&input(|i| {
            i.chat_port_answering = true;
        }));
        assert!(!d.start_chat);
        assert!(d.start_embedding);
        assert!(d.kill_allowed);
        assert!(!d.mark_embedding_ready);
    }

    #[test]
    fn chat_down_embedding_running_kills_and_starts_chat() {
        let d = decide(&input(|i| {
            i.embedding_port_answering = true;
        }));
        assert!(d.start_chat);
        assert!(!d.start_embedding);
        assert!(d.kill_allowed);
        assert!(!d.mark_embedding_ready);
    }

    #[test]
    fn both_down_kills_and_starts_both() {
        let d = decide(&input(|_| {}));
        assert!(d.start_chat);
        assert!(d.start_embedding);
        assert!(d.kill_allowed);
        assert!(!d.mark_embedding_ready);
    }

    #[test]
    fn chat_https_embedding_down_kills_and_starts_embedding() {
        let d = decide(&input(|i| {
            i.llm_url_is_https = true;
        }));
        assert!(!d.start_chat);
        assert!(d.start_embedding);
        assert!(d.kill_allowed);
        assert!(!d.mark_embedding_ready);
    }

    // ── managed-elsewhere: Lite edition, vector_db off (default) ───────

    #[test]
    fn managed_no_embedding_does_nothing() {
        let d = decide(&input(|i| {
            i.chat_managed_elsewhere = true;
            i.embedding_wanted = false;
        }));
        assert!(!d.start_chat);
        assert!(!d.start_embedding);
        assert!(!d.kill_allowed);
        assert!(!d.mark_embedding_ready);
    }

    #[test]
    fn managed_no_embedding_even_when_ports_free() {
        let d = decide(&input(|i| {
            i.chat_managed_elsewhere = true;
            i.embedding_wanted = false;
            // Both ports empty — must still do nothing.
        }));
        assert!(!d.start_chat);
        assert!(!d.start_embedding);
        assert!(!d.kill_allowed);
        assert!(!d.mark_embedding_ready);
    }

    // ── managed-elsewhere: Lite edition, vector_db ON ──────────────────

    #[test]
    fn managed_embedding_wanted_and_running_marks_ready() {
        let d = decide(&input(|i| {
            i.chat_managed_elsewhere = true;
            i.embedding_wanted = true;
            i.embedding_port_answering = true;
        }));
        assert!(!d.start_chat);
        assert!(!d.start_embedding);
        assert!(!d.kill_allowed);
        assert!(d.mark_embedding_ready);
    }

    #[test]
    fn managed_embedding_wanted_not_running_starts_without_kill() {
        let d = decide(&input(|i| {
            i.chat_managed_elsewhere = true;
            i.embedding_wanted = true;
        }));
        assert!(!d.start_chat);
        assert!(d.start_embedding);
        assert!(!d.kill_allowed);
        assert!(!d.mark_embedding_ready);
    }

    #[test]
    fn managed_embedding_https_marks_ready() {
        let d = decide(&input(|i| {
            i.chat_managed_elsewhere = true;
            i.embedding_wanted = true;
            i.embedding_url_is_https = true;
        }));
        assert!(!d.start_chat);
        assert!(!d.start_embedding);
        assert!(!d.kill_allowed);
        assert!(d.mark_embedding_ready);
    }

    #[test]
    fn managed_never_starts_chat_even_when_port_free() {
        let d = decide(&input(|i| {
            i.chat_managed_elsewhere = true;
            i.embedding_wanted = true;
            i.embedding_port_answering = true;
            // Chat port empty — must NOT start it.
        }));
        assert!(!d.start_chat);
        assert!(!d.kill_allowed);
    }

    #[test]
    fn managed_never_kills_even_when_embedding_needs_start() {
        let d = decide(&input(|i| {
            i.chat_managed_elsewhere = true;
            i.embedding_wanted = true;
            // Embedding not running, needs start.
        }));
        assert!(d.start_embedding);
        assert!(!d.kill_allowed, "kill would destroy the managed chat engine");
    }

    // ── hosted mode (Lite, vector_db off) ──────────────────────────────

    #[test]
    fn managed_hosted_no_embedding_does_nothing() {
        let d = decide(&input(|i| {
            i.chat_managed_elsewhere = true;
            i.llm_url_is_https = true;
            i.embedding_wanted = false;
            i.embedding_url_is_https = false;
        }));
        assert!(!d.start_chat);
        assert!(!d.start_embedding);
        assert!(!d.kill_allowed);
        assert!(!d.mark_embedding_ready);
    }
}
