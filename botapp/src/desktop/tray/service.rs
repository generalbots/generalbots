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

impl ServiceMonitor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: vec![
                ServiceStatus {
                    name: "API".to_string(),
                    running: false,
                    port: 8080,
                    url: "http://localhost:9000".to_string(),
                },
                ServiceStatus {
                    name: "UI".to_string(),
                    running: false,
                    port: 3000,
                    url: "http://localhost:3000".to_string(),
                },
            ],
        }
    }

    pub fn add_service(&mut self, name: &str, port: u16) {
        self.services.push(ServiceStatus {
            name: name.to_string(),
            running: false,
            port,
            url: format!("http://localhost:{port}"),
        });
    }

    pub async fn check_services(&mut self) -> Vec<ServiceStatus> {
        for service in &mut self.services {
            service.running = Self::check_service(&service.url).await;
        }
        self.services.clone()
    }

    pub async fn check_service(url: &str) -> bool {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }

        let Ok(client) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
        else {
            return false;
        };

        let health_url = format!("{}/health", url.trim_end_matches('/'));

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
