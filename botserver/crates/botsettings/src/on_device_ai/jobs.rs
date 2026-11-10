//! Background model downloads for the on-device AI page (#2109).
//!
//! One job per model, tracked in memory. The weights land in the local models
//! directory as `<file>.part` and are renamed only after verification, so a
//! cancelled or failed download keeps its partial file and resumes with a
//! `Range` request on the next start.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::catalog::ModelSpec;

pub(crate) const STATE_DOWNLOADING: &str = "downloading";
pub(crate) const STATE_DONE: &str = "done";
pub(crate) const STATE_CANCELLED: &str = "cancelled";
pub(crate) const STATE_FAILED: &str = "failed";

pub(crate) struct Job {
    pub id: String,
    pub model_id: &'static str,
    state: Mutex<&'static str>,
    bytes_done: AtomicU64,
    bytes_total: AtomicU64,
    /// Authoritative size from the HTTP response; 0 when the server sent no
    /// `Content-Length`, in which case only the checksum can verify the file.
    expected_bytes: AtomicU64,
    cancel: AtomicBool,
    started: Instant,
    error: Mutex<Option<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct JobSnapshot {
    pub job_id: String,
    pub model_id: &'static str,
    pub state: &'static str,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub speed_bps: u64,
    pub eta_secs: Option<u64>,
    pub error: Option<String>,
}

static JOBS: LazyLock<Mutex<HashMap<String, Arc<Job>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn state_of(job: &Job) -> &'static str {
    job.state.lock().map(|guard| *guard).unwrap_or(STATE_FAILED)
}

/// State read from a borrowed registry entry (`&Arc<Job>`); the field access
/// auto-derefs, so no reference juggling is needed inside `find` closures.
fn job_state(job: &Arc<Job>) -> &'static str {
    state_of(job)
}

fn set_state(job: &Job, state: &'static str) {
    if let Ok(mut guard) = job.state.lock() {
        *guard = state;
    }
}

fn fail(job: &Job, message: String) {
    log::warn!("[on_device_ai] download {} failed: {message}", job.model_id);
    if let Ok(mut guard) = job.error.lock() {
        *guard = Some(message);
    }
    set_state(job, STATE_FAILED);
}

/// Start (or join) a download for `spec`. Returns the job id.
pub(crate) fn start(spec: &'static ModelSpec, dir: &Path) -> Result<String, String> {
    let mut jobs = JOBS
        .lock()
        .map_err(|_| "job_registry_unavailable".to_string())?;
    if let Some(existing) = jobs
        .values()
        .find(|job| job.model_id == spec.id && job_state(job) == STATE_DOWNLOADING)
    {
        return Ok(existing.id.clone());
    }
    let id = Uuid::new_v4().to_string();
    let job = Arc::new(Job {
        id: id.clone(),
        model_id: spec.id,
        state: Mutex::new(STATE_DOWNLOADING),
        bytes_done: AtomicU64::new(0),
        bytes_total: AtomicU64::new(spec.size_bytes),
        expected_bytes: AtomicU64::new(0),
        cancel: AtomicBool::new(false),
        started: Instant::now(),
        error: Mutex::new(None),
    });
    jobs.insert(id.clone(), Arc::clone(&job));
    drop(jobs);
    let dir = dir.to_path_buf();
    tokio::spawn(async move {
        run(job, spec, dir).await;
    });
    Ok(id)
}

pub(crate) fn snapshot(job_id: &str) -> Option<JobSnapshot> {
    let jobs = JOBS.lock().ok()?;
    let job = jobs.get(job_id)?;
    Some(describe(job))
}

fn describe(job: &Job) -> JobSnapshot {
    let bytes_done = job.bytes_done.load(Ordering::Relaxed);
    let bytes_total = job.bytes_total.load(Ordering::Relaxed);
    let elapsed = job.started.elapsed().as_secs_f64().max(0.001);
    let speed_bps = (bytes_done as f64 / elapsed) as u64;
    let eta_secs = if speed_bps > 0 && bytes_total > bytes_done {
        Some((bytes_total - bytes_done) / speed_bps)
    } else {
        None
    };
    JobSnapshot {
        job_id: job.id.clone(),
        model_id: job.model_id,
        state: state_of(job),
        bytes_done,
        bytes_total,
        speed_bps,
        eta_secs,
        error: job.error.lock().ok().and_then(|guard| guard.clone()),
    }
}

pub(crate) fn cancel(job_id: &str) -> Result<(), String> {
    let jobs = JOBS
        .lock()
        .map_err(|_| "job_registry_unavailable".to_string())?;
    let job = jobs.get(job_id).ok_or_else(|| "job_not_found".to_string())?;
    job.cancel.store(true, Ordering::SeqCst);
    Ok(())
}

/// Id of the running download for `model_id`, so the page can resume polling
/// (and cancel) after a reload.
pub(crate) fn active_job_id(model_id: &str) -> Option<String> {
    let jobs = JOBS.lock().ok()?;
    jobs.values()
        .find(|job| job.model_id == model_id && job_state(job) == STATE_DOWNLOADING)
        .map(|job| job.id.clone())
}

/// Percentage of a running download for `model_id`, for the table row.
pub(crate) fn progress(model_id: &str) -> Option<u8> {
    let jobs = JOBS.lock().ok()?;
    let job = jobs
        .values()
        .find(|job| job.model_id == model_id && job_state(job) == STATE_DOWNLOADING)?;
    let total = job.bytes_total.load(Ordering::Relaxed);
    if total == 0 {
        return None;
    }
    let done = job.bytes_done.load(Ordering::Relaxed).min(total);
    Some(((done as f64 / total as f64) * 100.0).round().clamp(0.0, 100.0) as u8)
}

async fn run(job: Arc<Job>, spec: &'static ModelSpec, dir: PathBuf) {
    if let Err(error) = std::fs::create_dir_all(&dir) {
        fail(&job, format!("models_dir_unavailable: {error}"));
        return;
    }
    let part = dir.join(format!("{}.part", spec.file));
    let target = dir.join(spec.file);
    if target.exists() {
        let size = std::fs::metadata(&target).map(|meta| meta.len()).unwrap_or(0);
        job.bytes_done.store(size, Ordering::SeqCst);
        job.bytes_total.store(size, Ordering::SeqCst);
        set_state(&job, STATE_DONE);
        return;
    }

    let mut resume_from = std::fs::metadata(&part).map(|meta| meta.len()).unwrap_or(0);
    job.bytes_done.store(resume_from, Ordering::SeqCst);

    if !spec.repo.is_empty() {
        match fetch_into(&job, spec, &part, resume_from).await {
            Ok(()) => {}
            Err(DownloadError::Cancelled) => {
                set_state(&job, STATE_CANCELLED);
                return;
            }
            Err(DownloadError::Failed(message)) => {
                fail(&job, message);
                return;
            }
        }
        resume_from = std::fs::metadata(&part).map(|meta| meta.len()).unwrap_or(0);
    } else {
        fail(&job, "no_remote_source".to_string());
        return;
    }

    let expected = job.expected_bytes.load(Ordering::Relaxed);
    if expected > 0 && resume_from != expected {
        fail(&job, format!("size_mismatch: got {resume_from}, expected {expected}"));
        return;
    }
    if !spec.sha256.is_empty() {
        let part_for_hash = part.clone();
        match tokio::task::spawn_blocking(move || sha256_file(&part_for_hash)).await {
            Ok(Ok(digest)) if digest == spec.sha256 => {}
            Ok(Ok(digest)) => {
                let _ = std::fs::remove_file(&part);
                fail(&job, format!("checksum_mismatch: {digest}"));
                return;
            }
            Ok(Err(error)) => {
                fail(&job, format!("checksum_unreadable: {error}"));
                return;
            }
            Err(error) => {
                fail(&job, format!("checksum_task_failed: {error}"));
                return;
            }
        }
    }
    if let Err(error) = std::fs::rename(&part, &target) {
        fail(&job, format!("rename_failed: {error}"));
        return;
    }
    job.bytes_done.store(resume_from, Ordering::SeqCst);
    set_state(&job, STATE_DONE);
    log::info!("[on_device_ai] model {} ready at {}", spec.id, target.display());
}

enum DownloadError {
    Cancelled,
    Failed(String),
}

async fn fetch_into(
    job: &Job,
    spec: &'static ModelSpec,
    part: &Path,
    resume_from: u64,
) -> Result<(), DownloadError> {
    // No total timeout: a large GGUF takes far longer than any request budget.
    // Only the connection is bounded, so a dead endpoint fails fast.
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| DownloadError::Failed(format!("http_client_failed: {error}")))?;
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        spec.repo, spec.file
    );
    let mut request = client.get(&url);
    if resume_from > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={resume_from}-"));
    }
    let response = request
        .send()
        .await
        .map_err(|error| DownloadError::Failed(format!("request_failed: {error}")))?;

    let status = response.status();
    let append = resume_from > 0 && status == reqwest::StatusCode::PARTIAL_CONTENT;
    if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE && resume_from > 0 {
        // The partial file already holds every byte the server has.
        job.bytes_total.store(resume_from, Ordering::SeqCst);
        job.expected_bytes.store(resume_from, Ordering::SeqCst);
        return Ok(());
    }
    if !status.is_success() {
        return Err(DownloadError::Failed(format!("http_status_{}", status.as_u16())));
    }
    let offset = if append { resume_from } else { 0 };
    let content_length = response.content_length();
    if let Some(length) = content_length {
        job.expected_bytes.store(length + offset, Ordering::SeqCst);
    }
    let total = content_length
        .map(|length| length + offset)
        .unwrap_or(spec.size_bytes);
    job.bytes_total.store(total, Ordering::SeqCst);
    job.bytes_done.store(offset, Ordering::SeqCst);

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(part)
        .await
        .map_err(|error| DownloadError::Failed(format!("open_part_failed: {error}")))?;

    let mut response = response;
    loop {
        if job.cancel.load(Ordering::SeqCst) {
            return Err(DownloadError::Cancelled);
        }
        let chunk = response
            .chunk()
            .await
            .map_err(|error| DownloadError::Failed(format!("stream_failed: {error}")))?;
        let Some(chunk) = chunk else { break };
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|error| DownloadError::Failed(format!("write_failed: {error}")))?;
        job.bytes_done
            .fetch_add(chunk.len() as u64, Ordering::SeqCst);
    }
    tokio::io::AsyncWriteExt::flush(&mut file)
        .await
        .map_err(|error| DownloadError::Failed(format!("flush_failed: {error}")))?;
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
#[path = "jobs_tests.rs"]
mod jobs_tests;
