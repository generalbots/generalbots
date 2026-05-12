pub mod embedding_generator {
    use std::sync::atomic::{AtomicBool, Ordering};

    static EMBEDDING_SERVER_READY: AtomicBool = AtomicBool::new(false);

    pub fn set_embedding_server_ready(ready: bool) {
        EMBEDDING_SERVER_READY.store(ready, Ordering::SeqCst);
    }

    pub fn is_embedding_server_ready() -> bool {
        EMBEDDING_SERVER_READY.load(Ordering::SeqCst)
    }
}
