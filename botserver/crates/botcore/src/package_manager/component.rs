use std::collections::HashMap;

pub use botcorepkg::component::ComponentConfig;

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub component: String,
    pub container_name: String,
    pub container_ip: String,
    pub ports: Vec<u16>,
    pub env_vars: HashMap<String, String>,
    pub connection_info: String,
}
impl InstallResult {
    pub fn print(&self) {
        let component_upper = self.component.to_uppercase();
        println!("\n========================================");
        println!("  {component_upper} Installation Complete");
        println!("========================================\n");
        println!("Container: {}", self.container_name);
        println!("IP Address: {}", self.container_ip);
        println!("Ports: {:?}", self.ports);
        println!("\n--- Connection Info ---\n");
        println!("{}", self.connection_info);
        if !self.env_vars.is_empty() {
            println!("\n--- Environment Variables (.env) ---\n");
            for (key, value) in &self.env_vars {
                println!("{key}={value}");
            }
        }
        println!("\n========================================\n");
    }
}
