use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{fingerprint, Limits, Warm, WarmKey, WarmPool, WarmProcess};

struct Fake {
    alive: Arc<AtomicBool>,
    ended: Arc<AtomicUsize>,
}

impl WarmProcess for Fake {
    fn alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
    fn shut_down(self) {
        self.ended.fetch_add(1, Ordering::SeqCst);
    }
}

const LIMITS: Limits = Limits { idle: Duration::from_secs(600), max_age: Duration::from_secs(3600), per_user: 2 };

fn fake(ended: &Arc<AtomicUsize>) -> (Fake, Arc<AtomicBool>) {
    let alive = Arc::new(AtomicBool::new(true));
    (Fake { alive: alive.clone(), ended: ended.clone() }, alive)
}

fn warm(process: Fake, born: Instant) -> Warm<Fake> {
    Warm { process, born }
}

#[test]
fn a_kept_process_is_taken_by_the_next_turn_of_its_conversation_only() {
    let ended = Arc::new(AtomicUsize::new(0));
    let pool = WarmPool::new("test", LIMITS);
    let now = Instant::now();
    let (process, _) = fake(&ended);
    pool.put(WarmKey::new("vm-a", "c1"), "fp".into(), warm(process, now), now);
    assert!(pool.take(&WarmKey::new("vm-a", "c2"), "fp", now).is_none());
    assert!(pool.take(&WarmKey::new("vm-b", "c1"), "fp", now).is_none());
    assert!(pool.take(&WarmKey::new("vm-a", "c1"), "fp", now).is_some());
    assert!(pool.take(&WarmKey::new("vm-a", "c1"), "fp", now).is_none(), "a taken process is out of the pool");
    assert_eq!(ended.load(Ordering::SeqCst), 0);
}

#[test]
fn a_process_for_another_model_or_prompt_is_ended_not_reused() {
    let ended = Arc::new(AtomicUsize::new(0));
    let pool = WarmPool::new("test", LIMITS);
    let now = Instant::now();
    let (process, _) = fake(&ended);
    let key = WarmKey::new("vm-a", "c1");
    pool.put(key.clone(), fingerprint(&["gpt-5", "persona"]), warm(process, now), now);
    assert!(pool.take(&key, &fingerprint(&["gpt-5", "other"]), now).is_none());
    assert_eq!(ended.load(Ordering::SeqCst), 1);
    assert!(pool.is_empty());
}

#[test]
fn idle_too_long_old_or_dead_processes_are_reaped() {
    let ended = Arc::new(AtomicUsize::new(0));
    let pool = WarmPool::new("test", LIMITS);
    let now = Instant::now();
    let (idle, _) = fake(&ended);
    let (dead, dead_flag) = fake(&ended);
    let (fresh, _) = fake(&ended);
    pool.put(WarmKey::new("vm-a", "idle"), "fp".into(), warm(idle, now), now);
    pool.put(WarmKey::new("vm-b", "dead"), "fp".into(), warm(dead, now), now);
    dead_flag.store(false, Ordering::SeqCst);
    let later = now + Duration::from_secs(601);
    pool.put(WarmKey::new("vm-c", "fresh"), "fp".into(), warm(fresh, later), later);
    assert_eq!(pool.reap(later), 2);
    assert_eq!(ended.load(Ordering::SeqCst), 2);
    assert_eq!(pool.len(), 1);
    let old = later + Duration::from_secs(3600);
    assert!(pool.take(&WarmKey::new("vm-c", "fresh"), "fp", old).is_none(), "past its age, not reused");
    assert_eq!(ended.load(Ordering::SeqCst), 3);
}

#[test]
fn one_person_keeps_at_most_the_cap_and_the_longest_idle_goes() {
    let ended = Arc::new(AtomicUsize::new(0));
    let pool = WarmPool::new("test", LIMITS);
    let now = Instant::now();
    for (i, conversation) in ["c1", "c2", "c3"].iter().enumerate() {
        let at = now + Duration::from_secs(i as u64);
        let (process, _) = fake(&ended);
        pool.put(WarmKey::new("vm-a", conversation), "fp".into(), warm(process, at), at);
    }
    let (other, _) = fake(&ended);
    pool.put(WarmKey::new("vm-b", "c9"), "fp".into(), warm(other, now), now);
    assert_eq!(ended.load(Ordering::SeqCst), 1);
    let at = now + Duration::from_secs(5);
    assert!(pool.take(&WarmKey::new("vm-a", "c1"), "fp", at).is_none(), "the longest idle was ended");
    assert!(pool.take(&WarmKey::new("vm-a", "c3"), "fp", at).is_some());
    assert!(pool.take(&WarmKey::new("vm-b", "c9"), "fp", at).is_some(), "another person's process is kept");
}

#[test]
fn a_second_process_for_the_same_conversation_replaces_the_first() {
    let ended = Arc::new(AtomicUsize::new(0));
    let pool = WarmPool::new("test", LIMITS);
    let now = Instant::now();
    let (first, _) = fake(&ended);
    let (second, _) = fake(&ended);
    let key = WarmKey::new("vm-a", "c1");
    pool.put(key.clone(), "fp".into(), warm(first, now), now);
    pool.put(key, "fp".into(), warm(second, now), now);
    assert_eq!(ended.load(Ordering::SeqCst), 1);
    assert_eq!(pool.len(), 1);
}

#[test]
fn deleting_a_conversation_or_stopping_ends_its_processes() {
    let ended = Arc::new(AtomicUsize::new(0));
    let pool = WarmPool::new("test", LIMITS);
    let now = Instant::now();
    for (vm, conversation) in [("vm-a", "c1"), ("vm-a", "c2"), ("vm-b", "c3")] {
        let (process, _) = fake(&ended);
        pool.put(WarmKey::new(vm, conversation), "fp".into(), warm(process, now), now);
    }
    assert_eq!(pool.forget_conversation("c1"), 1);
    assert_eq!(pool.clear(), 2);
    assert_eq!(ended.load(Ordering::SeqCst), 3);
    assert!(pool.is_empty());
}

#[test]
fn a_dead_or_too_old_process_is_not_kept() {
    let ended = Arc::new(AtomicUsize::new(0));
    let pool = WarmPool::new("test", LIMITS);
    let now = Instant::now();
    let (dead, flag) = fake(&ended);
    flag.store(false, Ordering::SeqCst);
    pool.put(WarmKey::new("vm-a", "c1"), "fp".into(), warm(dead, now), now);
    let (old, _) = fake(&ended);
    pool.put(WarmKey::new("vm-a", "c2"), "fp".into(), warm(old, now), now + Duration::from_secs(3600));
    assert!(pool.is_empty());
    assert_eq!(ended.load(Ordering::SeqCst), 2);
}

#[test]
fn the_fingerprint_is_a_digest_not_the_text() {
    let digest = fingerprint(&["gpt-5", "a secret system prompt"]);
    assert_eq!(digest.len(), 16);
    assert!(!digest.contains("secret"));
    assert_eq!(digest, fingerprint(&["gpt-5", "a secret system prompt"]));
    assert_ne!(fingerprint(&["ab", "c"]), fingerprint(&["a", "bc"]), "parts are not just concatenated");
}
