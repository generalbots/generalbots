use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncusCloudConfig {
    pub cluster_name: String,
    pub nodes: Vec<IncusNode>,
    pub storage_pools: Vec<StoragePool>,
    pub networks: Vec<NetworkConfig>,
    pub profiles: Vec<ProfileConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncusNode {
    pub name: String,
    pub address: String,
    pub role: NodeRole,
    pub resources: NodeResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeRole {
    Controller,
    Worker,
    Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub storage_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePool {
    pub name: String,
    pub driver: String,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub type_: String,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub name: String,
    pub devices: HashMap<String, HashMap<String, String>>,
    pub config: HashMap<String, String>,
}

pub struct IncusCloudManager {
    config: IncusCloudConfig,
}

impl IncusCloudManager {
    pub fn new(config: IncusCloudConfig) -> Self {
        Self { config }
    }

    pub async fn bootstrap_cluster(&self) -> Result<()> {
        self.init_first_node().await?;
        self.setup_storage_pools().await?;
        self.setup_networks().await?;
        self.setup_profiles().await?;
        self.join_additional_nodes().await?;
        Ok(())
    }

    async fn init_first_node(&self) -> Result<()> {
        let first_node = &self.config.nodes[0];
        
        let output = AsyncCommand::new("incus")
            .args(&["admin", "init", "--auto"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to initialize Incus: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        AsyncCommand::new("incus")
            .args(&["config", "set", "cluster.https_address", &first_node.address])
            .output()
            .await?;

        AsyncCommand::new("incus")
            .args(&["config", "set", "core.https_address", &first_node.address])
            .output()
            .await?;

        Ok(())
    }

    async fn setup_storage_pools(&self) -> Result<()> {
        for pool in &self.config.storage_pools {
            let mut args = vec!["storage", "create", &pool.name, &pool.driver];
            
            for (key, value) in &pool.config {
                args.push(key);
                args.push(value);
            }

            AsyncCommand::new("incus")
                .args(&args)
                .output()
                .await?;
        }
        Ok(())
    }

    async fn setup_networks(&self) -> Result<()> {
        for network in &self.config.networks {
            let mut args = vec!["network", "create", &network.name, "--type", &network.type_];
            
            for (key, value) in &network.config {
                args.push("--config");
                args.push(&format!("{}={}", key, value));
            }

            AsyncCommand::new("incus")
                .args(&args)
                .output()
                .await?;
        }
        Ok(())
    }

    async fn setup_profiles(&self) -> Result<()> {
        for profile in &self.config.profiles {
            AsyncCommand::new("incus")
                .args(&["profile", "create", &profile.name])
                .output()
                .await?;

            for (key, value) in &profile.config {
                AsyncCommand::new("incus")
                    .args(&["profile", "set", &profile.name, key, value])
                    .output()
                    .await?;
            }

            for (device_name, device_config) in &profile.devices {
                let mut args = vec!["profile", "device", "add", &profile.name, device_name];
                for (key, value) in device_config {
                    args.push(key);
                    args.push(value);
                }
                AsyncCommand::new("incus")
                    .args(&args)
                    .output()
                    .await?;
            }
        }
        Ok(())
    }

    async fn join_additional_nodes(&self) -> Result<()> {
        if self.config.nodes.len() <= 1 {
            return Ok(());
        }

        let token_output = AsyncCommand::new("incus")
            .args(&["cluster", "add", "new-node"])
            .output()
            .await?;

        let token = String::from_utf8_lossy(&token_output.stdout).trim().to_string();

        for node in &self.config.nodes[1..] {
            self.join_node_to_cluster(&node.address, &token).await?;
        }

        Ok(())
    }

    async fn join_node_to_cluster(&self, node_address: &str, token: &str) -> Result<()> {
        AsyncCommand::new("ssh")
            .args(&[
                node_address,
                &format!("incus admin init --join-token {}", token)
            ])
            .output()
            .await?;

        Ok(())
    }

    pub async fn deploy_component(&self, component_name: &str, node_name: Option<&str>) -> Result<String> {
        let instance_name = format!("gb-{}-{}", component_name, uuid::Uuid::new_v4().to_string()[..8].to_string());
        
        let mut args = vec!["launch", "ubuntu:24.04", &instance_name, "--profile", "gbo"];
        
        if let Some(node) = node_name {
            args.extend(&["--target", node]);
        }

        let output = AsyncCommand::new("incus")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to launch instance: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        AsyncCommand::new("incus")
            .args(&["exec", &instance_name, "--", "cloud-init", "status", "--wait"])
            .output()
            .await?;

        self.setup_component_in_instance(&instance_name, component_name).await?;

        Ok(instance_name)
    }

    async fn setup_component_in_instance(&self, instance_name: &str, component_name: &str) -> Result<()> {
        let setup_script = format!(r#"
#!/bin/bash
set -e

# Update system
apt-get update -qq
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq wget curl unzip ca-certificates

# Create gbo directories
mkdir -p /opt/gbo/{{bin,data,conf,logs}}

# Create gbo user
useradd --system --no-create-home --shell /bin/false gbuser
chown -R gbuser:gbuser /opt/gbo

# Install component: {}
echo "Component {} setup complete"
"#, component_name, component_name);

        AsyncCommand::new("incus")
            .args(&["exec", instance_name, "--", "bash", "-c", &setup_script])
            .output()
            .await?;

        Ok(())
    }

    pub async fn create_vm(&self, vm_name: &str, template: &str) -> Result<String> {
        let output = AsyncCommand::new("incus")
            .args(&["launch", template, vm_name, "--vm", "--profile", "gbo-vm"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to create VM: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        Ok(vm_name.to_string())
    }

    pub async fn get_cluster_status(&self) -> Result<serde_json::Value> {
        let output = AsyncCommand::new("incus")
            .args(&["cluster", "list", "--format", "json"])
            .output()
            .await?;

        let status: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(status)
    }

    pub async fn get_instances(&self) -> Result<serde_json::Value> {
        let output = AsyncCommand::new("incus")
            .args(&["list", "--format", "json"])
            .output()
            .await?;

        let instances: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(instances)
    }

    pub async fn get_metrics(&self) -> Result<serde_json::Value> {
        let output = AsyncCommand::new("incus")
            .args(&["query", "/1.0/metrics"])
            .output()
            .await?;

        let metrics: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(metrics)
    }
}

pub fn create_default_cloud_config() -> IncusCloudConfig {
    IncusCloudConfig {
        cluster_name: "gbo-cloud".to_string(),
        nodes: vec![
            IncusNode {
                name: "controller-1".to_string(),
                address: "10.0.0.10:8443".to_string(),
                role: NodeRole::Controller,
                resources: NodeResources {
                    cpu_cores: 8,
                    memory_gb: 16,
                    storage_gb: 500,
                },
            }
        ],
        storage_pools: vec![
            StoragePool {
                name: "gbo-pool".to_string(),
                driver: "zfs".to_string(),
                config: HashMap::from([
                    ("size".to_string(), "100GB".to_string()),
                ]),
            }
        ],
        networks: vec![
            NetworkConfig {
                name: "gbo-net".to_string(),
                type_: "bridge".to_string(),
                config: HashMap::from([
                    ("ipv4.address".to_string(), "10.10.10.1/24".to_string()),
                    ("ipv4.nat".to_string(), "true".to_string()),
                ]),
            }
        ],
        profiles: vec![
            ProfileConfig {
                name: "gbo".to_string(),
                devices: HashMap::from([
                    ("eth0".to_string(), HashMap::from([
                        ("type".to_string(), "nic".to_string()),
                        ("network".to_string(), "gbo-net".to_string()),
                    ])),
                    ("root".to_string(), HashMap::from([
                        ("type".to_string(), "disk".to_string()),
                        ("pool".to_string(), "gbo-pool".to_string()),
                        ("path".to_string(), "/".to_string()),
                    ])),
                ]),
                config: HashMap::from([
                    ("security.privileged".to_string(), "true".to_string()),
                    ("limits.cpu".to_string(), "2".to_string()),
                    ("limits.memory".to_string(), "4GB".to_string()),
                ]),
            },
            ProfileConfig {
                name: "gbo-vm".to_string(),
                devices: HashMap::from([
                    ("eth0".to_string(), HashMap::from([
                        ("type".to_string(), "nic".to_string()),
                        ("network".to_string(), "gbo-net".to_string()),
                    ])),
                    ("root".to_string(), HashMap::from([
                        ("type".to_string(), "disk".to_string()),
                        ("pool".to_string(), "gbo-pool".to_string()),
                        ("path".to_string(), "/".to_string()),
                        ("size".to_string(), "20GB".to_string()),
                    ])),
                ]),
                config: HashMap::from([
                    ("limits.cpu".to_string(), "4".to_string()),
                    ("limits.memory".to_string(), "8GB".to_string()),
                ]),
            }
        ],
    }
}
