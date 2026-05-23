use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub active_sessions: u32,
    pub waiting_sessions: u32,
    pub avg_wait_time_secs: f64,
    pub max_wait_time_secs: f64,
    pub conversion_rate: f64,
    pub pipeline_value: f64,
    pub open_tickets: u32,
    pub tickets_resolved_today: u32,
    pub active_agents: u32,
    pub avg_satisfaction: f64,
    pub total_contacts: u64,
    pub total_leads: u64,
}

impl DashboardMetrics {
    pub fn zero() -> Self {
        Self {
            active_sessions: 0,
            waiting_sessions: 0,
            avg_wait_time_secs: 0.0,
            max_wait_time_secs: 0.0,
            conversion_rate: 0.0,
            pipeline_value: 0.0,
            open_tickets: 0,
            tickets_resolved_today: 0,
            active_agents: 0,
            avg_satisfaction: 0.0,
            total_contacts: 0,
            total_leads: 0,
        }
    }

    pub fn sample() -> Self {
        Self {
            active_sessions: 12,
            waiting_sessions: 3,
            avg_wait_time_secs: 45.5,
            max_wait_time_secs: 120.0,
            conversion_rate: 0.32,
            pipeline_value: 185_000.0,
            open_tickets: 7,
            tickets_resolved_today: 15,
            active_agents: 4,
            avg_satisfaction: 4.5,
            total_contacts: 234,
            total_leads: 89,
        }
    }

    pub fn format_wait_time(&self) -> String {
        if self.avg_wait_time_secs < 60.0 {
            format!("{:.0}s", self.avg_wait_time_secs)
        } else {
            let mins = (self.avg_wait_time_secs / 60.0).floor();
            let secs = self.avg_wait_time_secs % 60.0;
            format!("{:.0}m {:.0}s", mins, secs)
        }
    }

    pub fn format_conversion_rate(&self) -> String {
        format!("{:.1}%", self.conversion_rate * 100.0)
    }

    pub fn format_pipeline_value(&self) -> String {
        if self.pipeline_value >= 1_000_000.0 {
            format!("R$ {:.1}M", self.pipeline_value / 1_000_000.0)
        } else if self.pipeline_value >= 1_000.0 {
            format!("R$ {:.1}K", self.pipeline_value / 1_000.0)
        } else {
            format!("R$ {:.2}", self.pipeline_value)
        }
    }
}

pub fn get_dashboard_metrics() -> DashboardMetrics {
    DashboardMetrics::sample()
}

pub struct AgentPerformance {
    pub agent_id: String,
    pub name: String,
    pub sessions_handled: u32,
    pub avg_handle_time_secs: f64,
    pub satisfaction_score: f64,
    pub tickets_resolved: u32,
    pub is_online: bool,
}

impl AgentPerformance {
    pub fn new(agent_id: &str, name: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            name: name.to_string(),
            sessions_handled: 0,
            avg_handle_time_secs: 0.0,
            satisfaction_score: 0.0,
            tickets_resolved: 0,
            is_online: false,
        }
    }

    pub fn record_session(&mut self, duration_secs: f64, satisfaction: f64) {
        let total_time = self.avg_handle_time_secs * self.sessions_handled as f64;
        self.sessions_handled += 1;
        self.avg_handle_time_secs = (total_time + duration_secs) / self.sessions_handled as f64;
        self.satisfaction_score = ((self.satisfaction_score * (self.sessions_handled - 1) as f64) + satisfaction) / self.sessions_handled as f64;
    }

    pub fn resolve_ticket(&mut self) {
        self.tickets_resolved += 1;
    }
}

pub struct AdminReport {
    pub period_start: String,
    pub period_end: String,
    pub metrics: DashboardMetrics,
    pub top_agents: Vec<AgentPerformance>,
}

impl AdminReport {
    pub fn weekly() -> Self {
        Self {
            period_start: "2026-05-17".to_string(),
            period_end: "2026-05-23".to_string(),
            metrics: DashboardMetrics::sample(),
            top_agents: vec![
                AgentPerformance::new("agent1", "Alice"),
                AgentPerformance::new("agent2", "Bob"),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_zero() {
        let d = DashboardMetrics::zero();
        assert_eq!(d.active_sessions, 0);
        assert_eq!(d.pipeline_value, 0.0);
    }

    #[test]
    fn test_format_wait_time() {
        let mut d = DashboardMetrics::zero();
        d.avg_wait_time_secs = 45.0;
        assert_eq!(d.format_wait_time(), "45s");
        d.avg_wait_time_secs = 125.0;
        assert_eq!(d.format_wait_time(), "2m 5s");
    }

    #[test]
    fn test_agent_performance() {
        let mut a = AgentPerformance::new("a1", "Alice");
        assert_eq!(a.sessions_handled, 0);
        a.record_session(120.0, 4.5);
        assert_eq!(a.sessions_handled, 1);
        assert!((a.avg_handle_time_secs - 120.0).abs() < 0.01);
    }
}
