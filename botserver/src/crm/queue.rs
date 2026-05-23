use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AttendanceQueue {
    pub queue_id: Uuid,
    pub name: String,
    pub max_wait_time_secs: u32,
    pub agents: Vec<String>,
    queue: VecDeque<QueueEntry>,
    active_sessions: HashMap<Uuid, String>,
}

#[derive(Debug, Clone)]
struct QueueEntry {
    session_id: Uuid,
    enqueued_at: std::time::Instant,
    priority: u8,
}

impl AttendanceQueue {
    pub fn new(name: &str) -> Self {
        Self {
            queue_id: Uuid::new_v4(),
            name: name.to_string(),
            max_wait_time_secs: 300,
            agents: Vec::new(),
            queue: VecDeque::new(),
            active_sessions: HashMap::new(),
        }
    }

    pub fn with_max_wait(mut self, secs: u32) -> Self {
        self.max_wait_time_secs = secs;
        self
    }

    pub fn add_agent(&mut self, agent_id: &str) {
        if !self.agents.contains(&agent_id.to_string()) {
            self.agents.push(agent_id.to_string());
        }
    }

    pub fn remove_agent(&mut self, agent_id: &str) {
        self.agents.retain(|a| a != agent_id);
    }

    pub fn enqueue(&mut self, session_id: Uuid) {
        let priority = self.calculate_priority(&session_id);
        self.queue.push_back(QueueEntry {
            session_id,
            enqueued_at: std::time::Instant::now(),
            priority,
        });
    }

    pub fn dequeue(&mut self) -> Option<Uuid> {
        if self.agents.is_empty() {
            return None;
        }
        while let Some(entry) = self.queue.pop_front() {
            if entry.enqueued_at.elapsed().as_secs() > self.max_wait_time_secs as u64 {
                continue;
            }
            if self.active_sessions.contains_key(&entry.session_id) {
                continue;
            }
            return Some(entry.session_id);
        }
        None
    }

    pub fn assign_to_agent(&mut self, session_id: Uuid, agent_id: &str) {
        self.active_sessions.insert(session_id, agent_id.to_string());
    }

    pub fn release(&mut self, session_id: &Uuid) {
        self.active_sessions.remove(session_id);
    }

    pub fn waiting_count(&self) -> usize {
        self.queue.len()
    }

    pub fn active_count(&self) -> usize {
        self.active_sessions.len()
    }

    pub fn avg_wait_time_secs(&self) -> f64 {
        let now = std::time::Instant::now();
        let total: u64 = self.queue.iter()
            .map(|e| now.duration_since(e.enqueued_at).as_secs())
            .sum();
        if self.queue.is_empty() {
            0.0
        } else {
            total as f64 / self.queue.len() as f64
        }
    }

    fn calculate_priority(&self, _session_id: &Uuid) -> u8 {
        5
    }

    pub fn is_agent_available(&self, agent_id: &str) -> bool {
        self.agents.contains(&agent_id.to_string())
            && !self.active_sessions.values().any(|a| a == agent_id)
    }

    pub fn available_agents(&self) -> Vec<String> {
        self.agents.iter()
            .filter(|a| self.is_agent_available(a))
            .cloned()
            .collect()
    }
}

impl Default for AttendanceQueue {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_dequeue() {
        let mut q = AttendanceQueue::new("support");
        q.add_agent("agent1");
        let sid = Uuid::new_v4();
        q.enqueue(sid);
        assert_eq!(q.waiting_count(), 1);
        let dequeued = q.dequeue();
        assert_eq!(dequeued, Some(sid));
    }

    #[test]
    fn test_no_agents() {
        let mut q = AttendanceQueue::new("empty");
        q.enqueue(Uuid::new_v4());
        assert!(q.dequeue().is_none());
    }

    #[test]
    fn test_assign_release() {
        let mut q = AttendanceQueue::new("support");
        q.add_agent("agent1");
        let sid = Uuid::new_v4();
        q.enqueue(sid);
        let dequeued = q.dequeue().unwrap();
        q.assign_to_agent(dequeued, "agent1");
        assert_eq!(q.active_count(), 1);
        q.release(&sid);
        assert_eq!(q.active_count(), 0);
    }
}
