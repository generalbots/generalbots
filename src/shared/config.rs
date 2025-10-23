pub database: DatabaseConfig,
pub drive: DriveConfig,
pub meet: MeetConfig,
}

pub struct DatabaseConfig {
pub url: String,
pub max_connections: u32,
}

pub struct DriveConfig {
pub storage_path: String,
}

pub struct MeetConfig {
pub api_key: String,
pub api_secret: String,
}
use serde::Deserialize;
use dotenvy::dotenv;
use std::env;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
pub database: DatabaseConfig,
pub drive: DriveConfig,
pub meet: MeetConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
pub url: String,
pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct DriveConfig {
pub storage_path: String,
}

#[derive(Debug, Deserialize)]
pub struct MeetConfig {
pub api_key: String,
pub api_secret: String,
}

impl AppConfig {
pub fn load() -> anyhow::Result<Self> {
dotenv().ok();

Ok(Self {
database: DatabaseConfig {
url: env::var("DATABASE_URL")?,
max_connections: env::var("DATABASE_MAX_CONNECTIONS")?.parse()?,
},
drive: DriveConfig {
storage_path: env::var("DRIVE_STORAGE_PATH")?,
},
meet: MeetConfig {
api_key: env::var("MEET_API_KEY")?,
api_secret: env::var("MEET_API_SECRET")?,
},
})
}
}