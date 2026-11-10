//! Read-only service probes used by the tray/desktop status surfaces.

use serde::Serialize;

#[derive(Debug)]
pub struct ServiceMonitor {
    services: Vec<ServiceStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub port: u16,
    pub url: String,
}

/// One probed service. The URL always carries the port, so the status surface
/// can never report a port it is not actually checking.
fn service(name: &str, port: u16) -> ServiceStatus {
    ServiceStatus {
        name: name.to_string(),
        running: false,
        port,
        url: format!("http://localhost:{port}"),
    }
}

/// The `/health` URL for a service base URL, or `None` when it is not http(s).
fn health_url(url: &str) -> Option<String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }
    Some(format!("{}/health", url.trim_end_matches('/')))
}

impl ServiceMonitor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: vec![service("API", 8080), service("UI", 3000)],
        }
    }

    pub fn add_service(&mut self, name: &str, port: u16) {
        self.services.push(service(name, port));
    }

    pub async fn check_services(&mut self) -> Vec<ServiceStatus> {
        for service in &mut self.services {
            service.running = Self::check_service(&service.url).await;
        }
        self.services.clone()
    }

    pub async fn check_service(url: &str) -> bool {
        let Some(health_url) = health_url(url) else {
            return false;
        };

        let Ok(client) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
        else {
            return false;
        };

        client
            .get(&health_url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
    }

    #[must_use]
    pub fn get_service(&self, name: &str) -> Option<&ServiceStatus> {
        self.services.iter().find(|s| s.name == name)
    }

    #[must_use]
    pub fn all_running(&self) -> bool {
        self.services.iter().all(|s| s.running)
    }

    #[must_use]
    pub fn any_running(&self) -> bool {
        self.services.iter().any(|s| s.running)
    }
}

impl Default for ServiceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{health_url, ServiceMonitor};

    #[test]
    fn the_reported_port_is_the_port_being_probed() {
        let monitor = ServiceMonitor::new();
        let api = monitor.get_service("API").expect("the API service exists");
        assert_eq!(api.port, 8080);
        assert_eq!(api.url, "http://localhost:8080");
        assert_eq!(health_url(&api.url).as_deref(), Some("http://localhost:8080/health"));

        let ui = monitor.get_service("UI").expect("the UI service exists");
        assert_eq!(ui.port, 3000);
        assert_eq!(ui.url, "http://localhost:3000");
    }

    #[test]
    fn an_added_service_derives_its_url_from_the_port() {
        let mut monitor = ServiceMonitor::new();
        monitor.add_service("Vector", 6333);
        let added = monitor.get_service("Vector").expect("the added service exists");
        assert_eq!(added.port, 6333);
        assert_eq!(added.url, "http://localhost:6333");
        assert!(!added.running);
        assert!(!monitor.all_running());
        assert!(!monitor.any_running());
    }

    #[test]
    fn only_http_urls_get_a_health_probe() {
        assert_eq!(health_url("http://localhost:8080"), Some("http://localhost:8080/health".to_string()));
        assert_eq!(health_url("http://localhost:8080/"), Some("http://localhost:8080/health".to_string()));
        assert_eq!(health_url("https://gb.example"), Some("https://gb.example/health".to_string()));
        assert_eq!(health_url("ftp://localhost:8080"), None);
        assert_eq!(health_url("file:///etc/passwd"), None);
        assert_eq!(health_url("localhost:8080"), None);
        assert_eq!(health_url(""), None);
    }
}
