use std::collections::HashMap;
use std::path::PathBuf;
use super::command_guard::SafeCommand;
use super::command_validation::{validate_argument, CommandGuardError};

pub fn safe_pdftotext(
    pdf_path: &std::path::Path,
    _allowed_paths: &[PathBuf],
) -> Result<String, CommandGuardError> {
    let output = SafeCommand::new("pdftotext")?
        .allow_path(
            pdf_path
                .parent()
                .unwrap_or(std::path::Path::new("/tmp"))
                .to_path_buf(),
        )
        .arg("-layout")?
        .path_arg(pdf_path)?
        .arg("-")?
        .execute()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(CommandGuardError::ExecutionFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub async fn safe_pdftotext_async(
    pdf_path: &std::path::Path,
) -> Result<String, CommandGuardError> {
    let parent = pdf_path
        .parent()
        .unwrap_or(std::path::Path::new("/tmp"))
        .to_path_buf();

    let output = SafeCommand::new("pdftotext")?
        .allow_path(parent)
        .arg("-layout")?
        .path_arg(pdf_path)?
        .arg("-")?
        .execute_async()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(CommandGuardError::ExecutionFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub async fn safe_pandoc_async(
    input_path: &std::path::Path,
    from_format: &str,
    to_format: &str,
) -> Result<String, CommandGuardError> {
    validate_argument(from_format)?;
    validate_argument(to_format)?;

    let allowed_formats = ["docx", "plain", "html", "markdown", "rst", "latex", "txt"];
    if !allowed_formats.contains(&from_format) || !allowed_formats.contains(&to_format) {
        return Err(CommandGuardError::InvalidArgument(
            "Invalid format specified".to_string(),
        ));
    }

    let parent = input_path
        .parent()
        .unwrap_or(std::path::Path::new("/tmp"))
        .to_path_buf();

    let output = SafeCommand::new("pandoc")?
        .allow_path(parent)
        .arg("-f")?
        .arg(from_format)?
        .arg("-t")?
        .arg(to_format)?
        .path_arg(input_path)?
        .execute_async()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(CommandGuardError::ExecutionFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub fn safe_nvidia_smi() -> Result<HashMap<String, f32>, CommandGuardError> {
    let output = SafeCommand::new("nvidia-smi")?
        .arg("--query-gpu=utilization.gpu,utilization.memory")?
        .arg("--format=csv,noheader,nounits")?
        .execute()?;

    if !output.status.success() {
        return Err(CommandGuardError::ExecutionFailed(
            "Failed to query GPU utilization".to_string(),
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut util = HashMap::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            util.insert(
                "gpu".to_string(),
                parts[0].trim().parse::<f32>().unwrap_or_default(),
            );
            util.insert(
                "memory".to_string(),
                parts[1].trim().parse::<f32>().unwrap_or_default(),
            );
        }
    }

    Ok(util)
}

pub fn has_nvidia_gpu_safe() -> bool {
    SafeCommand::new("nvidia-smi")
        .and_then(|cmd| {
            cmd.arg("--query-gpu=utilization.gpu")?
                .arg("--format=csv,noheader,nounits")
        })
        .and_then(|cmd| cmd.execute())
        .map(|output| output.status.success())
        .unwrap_or(false)
}
