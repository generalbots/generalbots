use crate::queue_types::AttendantCSV;
use log::{error, info, warn};
use std::path::PathBuf;
use uuid::Uuid;

pub fn is_transfer_enabled(bot_id: Uuid, work_path: &str) -> bool {
    let config_path = PathBuf::from(work_path)
        .join(format!("{}.gbai", bot_id))
        .join("config.csv");

    if !config_path.exists() {
        let alt_path = PathBuf::from(work_path).join("config.csv");
        if alt_path.exists() {
            return check_config_for_crm_enabled(&alt_path);
        }
        warn!("Config file not found: {}", config_path.display());
        return false;
    }

    check_config_for_crm_enabled(&config_path)
}

fn check_config_for_crm_enabled(config_path: &PathBuf) -> bool {
    match std::fs::read_to_string(config_path) {
        Ok(content) => {
            for line in content.lines() {
                let line_lower = line.to_lowercase();
                if (line_lower.contains("crm-enabled") || line_lower.contains("crm_enabled"))
                    && line_lower.contains("true")
                {
                    info!("CRM enabled via crm-enabled setting");
                    return true;
                }
                if line_lower.contains("transfer") && line_lower.contains("true") {
                    info!("CRM enabled via legacy transfer setting");
                    return true;
                }
            }
            false
        }
        Err(e) => {
            error!("Failed to read config file: {}", e);
            false
        }
    }
}

pub fn read_attendants_csv(bot_id: Uuid, work_path: &str) -> Vec<AttendantCSV> {
    let attendant_path = PathBuf::from(work_path)
        .join(format!("{}.gbai", bot_id))
        .join("attendant.csv");

    if !attendant_path.exists() {
        warn!("Attendant file not found: {}", attendant_path.display());
        return Vec::new();
    }

    match std::fs::read_to_string(&attendant_path) {
        Ok(content) => {
            let mut attendants = Vec::new();
            let mut lines = content.lines();
            lines.next();
            for line in lines {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 4 {
                    attendants.push(AttendantCSV {
                        id: parts[0].to_string(),
                        name: parts[1].to_string(),
                        channel: parts[2].to_string(),
                        preferences: parts[3].to_string(),
                        department: parts.get(4).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                        aliases: parts.get(5).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                        phone: parts.get(6).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                        email: parts.get(7).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                        teams: parts.get(8).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                        google: parts.get(9).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                    });
                }
            }
            attendants
        }
        Err(e) => {
            error!("Failed to read attendant file: {}", e);
            Vec::new()
        }
    }
}

pub fn find_attendant_by_identifier(
    bot_id: Uuid,
    work_path: &str,
    identifier: &str,
) -> Option<AttendantCSV> {
    let attendants = read_attendants_csv(bot_id, work_path);
    let identifier_lower = identifier.to_lowercase().trim().to_string();
    for att in attendants {
        if att.id.to_lowercase() == identifier_lower {
            return Some(att);
        }
        if att.name.to_lowercase() == identifier_lower {
            return Some(att);
        }
        if let Some(ref phone) = att.phone {
            let phone_normalized: String = phone.chars().filter(|c| c.is_numeric() || *c == '+').collect();
            let id_normalized: String = identifier.chars().filter(|c| c.is_numeric() || *c == '+').collect();
            if phone_normalized == id_normalized || phone.to_lowercase() == identifier_lower {
                return Some(att);
            }
        }
        if let Some(ref email) = att.email {
            if email.to_lowercase() == identifier_lower {
                return Some(att);
            }
        }
        if let Some(ref teams) = att.teams {
            if teams.to_lowercase() == identifier_lower {
                return Some(att);
            }
        }
        if let Some(ref google) = att.google {
            if google.to_lowercase() == identifier_lower {
                return Some(att);
            }
        }
        if let Some(ref aliases) = att.aliases {
            for alias in aliases.split(';') {
                if alias.trim().to_lowercase() == identifier_lower {
                    return Some(att);
                }
            }
        }
    }
    None
}

pub fn find_attendants_by_channel(
    bot_id: Uuid,
    work_path: &str,
    channel: &str,
) -> Vec<AttendantCSV> {
    let attendants = read_attendants_csv(bot_id, work_path);
    let channel_lower = channel.to_lowercase();
    attendants
        .into_iter()
        .filter(|att| att.channel.to_lowercase() == "all" || att.channel.to_lowercase() == channel_lower)
        .collect()
}

pub fn find_attendants_by_department(
    bot_id: Uuid,
    work_path: &str,
    department: &str,
) -> Vec<AttendantCSV> {
    let attendants = read_attendants_csv(bot_id, work_path);
    let dept_lower = department.to_lowercase();
    attendants
        .into_iter()
        .filter(|att| att.department.as_ref().map(|d| d.to_lowercase() == dept_lower).unwrap_or(false))
        .collect()
}
