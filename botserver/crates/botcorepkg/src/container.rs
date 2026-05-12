#[derive(Debug, Clone)]
pub struct NatRule {
    pub port: u16,
    pub protocol: String,
}

impl NatRule {
    pub fn new(port: u16, protocol: &str) -> Self {
        Self {
            port,
            protocol: protocol.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerSettings {
    pub container_name: String,
    pub ip: String,
    pub user: String,
    pub group: Option<String>,
    pub working_dir: Option<String>,
    pub service_template: String,
    pub nat_rules: Vec<NatRule>,
    pub binary_path: String,
    pub config_path: String,
    pub data_path: Option<String>,
    pub exec_cmd_args: Vec<String>,
    pub internal_ports: Vec<u16>,
    pub external_port: Option<u16>,
}

impl ContainerSettings {
    pub fn new(
        container_name: &str,
        ip: &str,
        user: &str,
        binary_path: &str,
        config_path: &str,
    ) -> Self {
        Self {
            container_name: container_name.to_string(),
            ip: ip.to_string(),
            user: user.to_string(),
            group: None,
            working_dir: None,
            service_template: String::new(),
            nat_rules: Vec::new(),
            binary_path: binary_path.to_string(),
            config_path: config_path.to_string(),
            data_path: None,
            exec_cmd_args: Vec::new(),
            internal_ports: Vec::new(),
            external_port: None,
        }
    }

    pub fn with_group(mut self, group: &str) -> Self {
        self.group = Some(group.to_string());
        self
    }

    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    pub fn with_service_template(mut self, template: &str) -> Self {
        self.service_template = template.to_string();
        self
    }

    pub fn with_nat_rules(mut self, rules: Vec<NatRule>) -> Self {
        self.nat_rules = rules;
        self
    }

    pub fn with_data_path(mut self, path: &str) -> Self {
        self.data_path = Some(path.to_string());
        self
    }

    pub fn with_exec_args(mut self, args: Vec<String>) -> Self {
        self.exec_cmd_args = args;
        self
    }

    pub fn with_internal_ports(mut self, ports: Vec<u16>) -> Self {
        self.internal_ports = ports;
        self
    }
}
