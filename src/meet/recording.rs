
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::core::shared::schema::meeting_recordings;
use crate::core::shared::utils::DbPool;
use crate::core::shared::{format_timestamp_plain, format_timestamp_srt, format_timestamp_vtt};

#[derive(Debug, Clone)]
pub enum RecordingError {
    DatabaseError(String),
    NotFound,
    AlreadyExists,
    AlreadyRecording,
    InvalidState(String),
    StorageError(String),
    TranscriptionError(String),
    TranscriptionNotReady,
    UnsupportedLanguage(String),
    ExportFailed(String),
    Unauthorized,
}

impl std::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "Database error: {e}"),
            Self::NotFound => write!(f, "Recording not found"),
            Self::AlreadyExists => write!(f, "Recording already exists"),
            Self::AlreadyRecording => write!(f, "Already recording"),
            Self::InvalidState(s) => write!(f, "Invalid state: {s}"),
            Self::StorageError(e) => write!(f, "Storage error: {e}"),
            Self::TranscriptionError(e) => write!(f, "Transcription error: {e}"),
            Self::TranscriptionNotReady => write!(f, "Transcription not ready"),
            Self::UnsupportedLanguage(l) => write!(f, "Unsupported language: {l}"),
            Self::ExportFailed(e) => write!(f, "Export failed: {e}"),
            Self::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl std::error::Error for RecordingError {}

use super::webinar::{
    RecordingQuality, RecordingStatus, TranscriptionFormat, TranscriptionSegment,
    TranscriptionStatus, TranscriptionWord, WebinarRecording, WebinarTranscription,
};

const MAX_RECORDING_DURATION_SECONDS: u64 = 28800;

const DEFAULT_TRANSCRIPTION_LANGUAGE: &str = "en-US";

const SUPPORTED_LANGUAGES: &[&str] = &[
    "en-US", "en-GB", "es-ES", "es-MX", "fr-FR", "de-DE", "it-IT", "pt-BR", "pt-PT", "nl-NL",
    "pl-PL", "ru-RU", "ja-JP", "ko-KR", "zh-CN", "zh-TW", "ar-SA", "hi-IN", "tr-TR", "vi-VN",
];


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    pub max_duration_seconds: u64,
    pub default_quality: RecordingQuality,
    pub storage_backend: StorageBackend,
    pub storage_bucket: String,
    pub auto_transcribe: bool,
    pub default_language: String,
    pub transcription_provider: TranscriptionProvider,
    pub retention_days: u32,
    pub speaker_diarization: bool,
    pub max_speakers: u8,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            max_duration_seconds: MAX_RECORDING_DURATION_SECONDS,
            default_quality: RecordingQuality::Standard,
            storage_backend: StorageBackend::Local,
            storage_bucket: "recordings".to_string(),
            auto_transcribe: true,
            default_language: DEFAULT_TRANSCRIPTION_LANGUAGE.to_string(),
            transcription_provider: TranscriptionProvider::Whisper,
            retention_days: 90,
            speaker_diarization: true,
            max_speakers: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum StorageBackend {
    #[default]
    Local,
    S3,
    Azure,
    Gcs,
}

impl std::fmt::Display for StorageBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageBackend::Local => write!(f, "local"),
            StorageBackend::S3 => write!(f, "s3"),
            StorageBackend::Azure => write!(f, "azure"),
            StorageBackend::Gcs => write!(f, "gcs"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum TranscriptionProvider {
    #[default]
    Whisper,
    AzureSpeech,
    GoogleSpeech,
    AwsTranscribe,
    DeepGram,
    AssemblyAI,
}

impl std::fmt::Display for TranscriptionProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptionProvider::Whisper => write!(f, "whisper"),
            TranscriptionProvider::AzureSpeech => write!(f, "azure_speech"),
            TranscriptionProvider::GoogleSpeech => write!(f, "google_speech"),
            TranscriptionProvider::AwsTranscribe => write!(f, "aws_transcribe"),
            TranscriptionProvider::DeepGram => write!(f, "deepgram"),
            TranscriptionProvider::AssemblyAI => write!(f, "assembly_ai"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub status: RecordingStatus,
    pub quality: RecordingQuality,
    pub started_at: DateTime<Utc>,
    pub paused_at: Option<DateTime<Utc>>,
    pub total_paused_duration_ms: u64,
    pub audio_track_id: Option<String>,
    pub video_track_id: Option<String>,
    pub screen_share_track_id: Option<String>,
    pub file_path: Option<String>,
    pub chunk_count: u32,
    pub bytes_written: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionJob {
    pub id: Uuid,
    pub recording_id: Uuid,
    pub webinar_id: Uuid,
    pub status: TranscriptionStatus,
    pub language: String,
    pub provider: TranscriptionProvider,
    pub enable_speaker_diarization: bool,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress_percent: u8,
    pub error_message: Option<String>,
    pub retry_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordingEvent {
    Started {
        recording_id: Uuid,
        webinar_id: Uuid,
    },
    Paused {
        recording_id: Uuid,
    },
    Resumed {
        recording_id: Uuid,
    },
    Stopped {
        recording_id: Uuid,
        duration_seconds: u64,
    },
    ChunkWritten {
        recording_id: Uuid,
        chunk_number: u32,
        bytes: u64,
    },
    ProcessingStarted {
        recording_id: Uuid,
    },
    ProcessingCompleted {
        recording_id: Uuid,
        file_url: String,
    },
    ProcessingFailed {
        recording_id: Uuid,
        error: String,
    },
    TranscriptionStarted {
        transcription_id: Uuid,
        recording_id: Uuid,
    },
    TranscriptionProgress {
        transcription_id: Uuid,
        progress_percent: u8,
    },
    TranscriptionSegmentReady {
        transcription_id: Uuid,
        segment: TranscriptionSegment,
    },
    TranscriptionCompleted {
        transcription_id: Uuid,
        word_count: u32,
    },
    TranscriptionFailed {
        transcription_id: Uuid,
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRecordingRequest {
    pub webinar_id: Uuid,
    pub quality: Option<RecordingQuality>,
    pub enable_transcription: Option<bool>,
    pub transcription_language: Option<String>,
    pub speaker_diarization: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopRecordingRequest {
    pub recording_id: Uuid,
    pub start_transcription: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTranscriptionRequest {
    pub format: TranscriptionFormat,
    pub include_timestamps: bool,
    pub include_speaker_names: bool,
    pub max_line_length: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTranscriptionResponse {
    pub format: TranscriptionFormat,
    pub content: String,
    pub content_type: String,
    pub filename: String,
}

pub struct RecordingService {
    pool: DbPool,
    config: RecordingConfig,
    active_sessions: Arc<RwLock<HashMap<Uuid, RecordingSession>>>,
    transcription_jobs: Arc<RwLock<HashMap<Uuid, TranscriptionJob>>>,
    event_sender: broadcast::Sender<RecordingEvent>,
}

impl RecordingService {
    pub fn new(pool: DbPool, config: RecordingConfig) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        Self {
            pool,
            config,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            transcription_jobs: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RecordingEvent> {
        self.event_sender.subscribe()
    }

    pub async fn start_recording(
        &self,
        request: StartRecordingRequest,
    ) -> Result<WebinarRecording, RecordingError> {
        // Check if webinar is already being recorded
        let sessions = self.active_sessions.read().await;
        if sessions.values().any(|s| s.webinar_id == request.webinar_id) {
            return Err(RecordingError::AlreadyRecording);
        }
        drop(sessions);

        let recording_id = Uuid::new_v4();
        let now = Utc::now();
        let quality = request.quality.unwrap_or(self.config.default_quality.clone());

        // Create recording session
        let session = RecordingSession {
            id: recording_id,
            webinar_id: request.webinar_id,
            status: RecordingStatus::Recording,
            quality: quality.clone(),
            started_at: now,
            paused_at: None,
            total_paused_duration_ms: 0,
            audio_track_id: None,
            video_track_id: None,
            screen_share_track_id: None,
            file_path: Some(format!(
                "{}/{}/{}.webm",
                self.config.storage_bucket,
                request.webinar_id,
                recording_id
            )),
            chunk_count: 0,
            bytes_written: 0,
        };

        // Store session
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(recording_id, session);
        drop(sessions);

        // Create database record
        self.create_recording_record(recording_id, request.webinar_id, &quality, now)
            .await?;

        // Broadcast event
        let _ = self.event_sender.send(RecordingEvent::Started {
            recording_id,
            webinar_id: request.webinar_id,
        });

        Ok(WebinarRecording {
            id: recording_id,
            webinar_id: request.webinar_id,
            status: RecordingStatus::Recording,
            duration_seconds: 0,
            file_size_bytes: 0,
            file_url: None,
            download_url: None,
            quality,
            started_at: now,
            ended_at: None,
            processed_at: None,
            expires_at: if self.config.retention_days > 0 {
                Some(now + chrono::Duration::days(self.config.retention_days as i64))
            } else {
                None
            },
            view_count: 0,
            download_count: 0,
        })
    }

    pub async fn pause_recording(&self, recording_id: Uuid) -> Result<(), RecordingError> {
        let mut sessions = self.active_sessions.write().await;
        let session = sessions
            .get_mut(&recording_id)
            .ok_or(RecordingError::NotFound)?;

        if session.status != RecordingStatus::Recording {
            return Err(RecordingError::InvalidState(
                "Recording is not active".to_string(),
            ));
        }

        session.paused_at = Some(Utc::now());

        let _ = self.event_sender.send(RecordingEvent::Paused { recording_id });

        Ok(())
    }

    pub async fn resume_recording(&self, recording_id: Uuid) -> Result<(), RecordingError> {
        let mut sessions = self.active_sessions.write().await;
        let session = sessions
            .get_mut(&recording_id)
            .ok_or(RecordingError::NotFound)?;

        if let Some(paused_at) = session.paused_at {
            let pause_duration = (Utc::now() - paused_at).num_milliseconds() as u64;
            session.total_paused_duration_ms += pause_duration;
            session.paused_at = None;
        }

        let _ = self.event_sender.send(RecordingEvent::Resumed { recording_id });

        Ok(())
    }

    pub async fn stop_recording(
        &self,
        request: StopRecordingRequest,
    ) -> Result<WebinarRecording, RecordingError> {
        let mut sessions = self.active_sessions.write().await;
        let session = sessions
            .remove(&request.recording_id)
            .ok_or(RecordingError::NotFound)?;
        drop(sessions);

        let now = Utc::now();
        let duration_seconds =
            ((now - session.started_at).num_milliseconds() as u64 - session.total_paused_duration_ms)
                / 1000;

        // Update database record
        self.update_recording_stopped(
            request.recording_id,
            now,
            duration_seconds,
            session.bytes_written,
        )
        .await?;

        // Broadcast stop event
        let _ = self.event_sender.send(RecordingEvent::Stopped {
            recording_id: request.recording_id,
            duration_seconds,
        });

        // Start processing
        self.process_recording(request.recording_id).await?;

        // Start transcription if requested
        if request.start_transcription.unwrap_or(self.config.auto_transcribe) {
            self.start_transcription(request.recording_id, session.webinar_id, None)
                .await?;
        }

        Ok(WebinarRecording {
            id: request.recording_id,
            webinar_id: session.webinar_id,
            status: RecordingStatus::Processing,
            duration_seconds,
            file_size_bytes: session.bytes_written,
            file_url: None,
            download_url: None,
            quality: session.quality,
            started_at: session.started_at,
            ended_at: Some(now),
            processed_at: None,
            expires_at: if self.config.retention_days > 0 {
                Some(now + chrono::Duration::days(self.config.retention_days as i64))
            } else {
                None
            },
            view_count: 0,
            download_count: 0,
        })
    }

    async fn process_recording(&self, recording_id: Uuid) -> Result<(), RecordingError> {
        let _ = self
            .event_sender
            .send(RecordingEvent::ProcessingStarted { recording_id });

        // In production, this would:
        // 1. Convert raw recording to final format (MP4/WebM)
        // 2. Generate multiple quality versions
        // 3. Generate thumbnails
        // 4. Upload to cloud storage
        // 5. Update database with URLs

        // Simulate processing completion
        let file_url = format!(
            "https://storage.example.com/recordings/{}.mp4",
            recording_id
        );

        let download_url = format!(
            "https://storage.example.com/recordings/{}/download",
            recording_id
        );
        self.update_recording_processed(recording_id, &file_url, &download_url)
            .await?;

        let _ = self
            .event_sender
            .send(RecordingEvent::ProcessingCompleted {
                recording_id,
                file_url,
            });

        Ok(())
    }

    pub async fn start_transcription(
        &self,
        recording_id: Uuid,
        webinar_id: Uuid,
        language: Option<String>,
    ) -> Result<WebinarTranscription, RecordingError> {
        let transcription_id = Uuid::new_v4();
        let now = Utc::now();
        let language = language.unwrap_or_else(|| self.config.default_language.clone());

        // Validate language
        if !SUPPORTED_LANGUAGES.contains(&language.as_str()) {
            return Err(RecordingError::UnsupportedLanguage(language));
        }

        // Create transcription job
        let job = TranscriptionJob {
            id: transcription_id,
            recording_id,
            webinar_id,
            status: TranscriptionStatus::Pending,
            language: language.clone(),
            provider: self.config.transcription_provider.clone(),
            enable_speaker_diarization: self.config.speaker_diarization,
            created_at: now,
            started_at: None,
            completed_at: None,
            progress_percent: 0,
            error_message: None,
            retry_count: 0,
        };

        let mut jobs = self.transcription_jobs.write().await;
        jobs.insert(transcription_id, job);
        drop(jobs);

        // Create database record
        self.create_transcription_record(transcription_id, recording_id, &language)
            .await?;

        // Start transcription process (async)
        let service = self.clone_for_task();
        tokio::spawn(async move {
            service
                .run_transcription(transcription_id, recording_id)
                .await
        });

        let _ = self
            .event_sender
            .send(RecordingEvent::TranscriptionStarted {
                transcription_id,
                recording_id,
            });

        Ok(WebinarTranscription {
            id: transcription_id,
            webinar_id,
            recording_id,
            status: TranscriptionStatus::Pending,
            language,
            duration_seconds: 0,
            word_count: 0,
            speaker_count: 0,
            segments: vec![],
            full_text: None,
            vtt_url: None,
            srt_url: None,
            json_url: None,
            created_at: now,
            completed_at: None,
            confidence_score: 0.0,
        })
    }

    async fn run_transcription(&self, transcription_id: Uuid, recording_id: Uuid) {
        log::info!("Starting transcription {transcription_id} for recording {recording_id}");
        // Update status to in progress
        {
            let mut jobs = self.transcription_jobs.write().await;
            if let Some(job) = jobs.get_mut(&transcription_id) {
                job.status = TranscriptionStatus::InProgress;
                job.started_at = Some(Utc::now());
            }
        }

        // In production, this would:
        // 1. Download/access the recording file
        // 2. Extract audio track
        // 3. Send to transcription provider (Whisper, Azure, etc.)
        // 4. Process results with speaker diarization
        // 5. Store segments in database
        // 6. Generate VTT/SRT files

        // Simulate transcription progress
        for progress in (0..=100).step_by(10) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            {
                let mut jobs = self.transcription_jobs.write().await;
                if let Some(job) = jobs.get_mut(&transcription_id) {
                    job.progress_percent = progress as u8;
                }
            }

            let _ = self
                .event_sender
                .send(RecordingEvent::TranscriptionProgress {
                    transcription_id,
                    progress_percent: progress as u8,
                });

            // Emit segment at 50%
            if progress == 50 {
                let segment = TranscriptionSegment {
                    id: Uuid::new_v4(),
                    start_time_ms: 0,
                    end_time_ms: 5000,
                    text: "Welcome to this webinar session.".to_string(),
                    speaker_id: Some("speaker_1".to_string()),
                    speaker_name: Some("Host".to_string()),
                    confidence: 0.95,
                    words: vec![
                        TranscriptionWord {
                            word: "Welcome".to_string(),
                            start_time_ms: 0,
                            end_time_ms: 500,
                            confidence: 0.98,
                        },
                        TranscriptionWord {
                            word: "to".to_string(),
                            start_time_ms: 500,
                            end_time_ms: 700,
                            confidence: 0.99,
                        },
                        TranscriptionWord {
                            word: "this".to_string(),
                            start_time_ms: 700,
                            end_time_ms: 900,
                            confidence: 0.97,
                        },
                        TranscriptionWord {
                            word: "webinar".to_string(),
                            start_time_ms: 900,
                            end_time_ms: 1500,
                            confidence: 0.96,
                        },
                        TranscriptionWord {
                            word: "session".to_string(),
                            start_time_ms: 1500,
                            end_time_ms: 2000,
                            confidence: 0.94,
                        },
                    ],
                };

                let _ = self
                    .event_sender
                    .send(RecordingEvent::TranscriptionSegmentReady {
                        transcription_id,
                        segment,
                    });
            }
        }

        // Mark as completed
        {
            let mut jobs = self.transcription_jobs.write().await;
            if let Some(job) = jobs.get_mut(&transcription_id) {
                job.status = TranscriptionStatus::Completed;
                job.completed_at = Some(Utc::now());
                job.progress_percent = 100;
            }
        }

        // Create mock transcription data
        let full_text = "Welcome to this webinar session.".to_string();
        let segments: Vec<TranscriptionSegment> = vec![];

        // Update database
        let _ = self
            .update_transcription_completed(transcription_id, &full_text, &segments)
            .await;

        let _ = self
            .event_sender
            .send(RecordingEvent::TranscriptionCompleted {
                transcription_id,
                word_count: 1500,
            });
    }

    pub async fn get_recording(&self, recording_id: Uuid) -> Result<WebinarRecording, RecordingError> {
        // Check active sessions first
        let sessions = self.active_sessions.read().await;
        if let Some(session) = sessions.get(&recording_id) {
            let duration_seconds = if session.paused_at.is_some() {
                0 // Paused
            } else {
                ((Utc::now() - session.started_at).num_milliseconds() as u64
                    - session.total_paused_duration_ms)
                    / 1000
            };

            return Ok(WebinarRecording {
                id: session.id,
                webinar_id: session.webinar_id,
                status: session.status.clone(),
                duration_seconds,
                file_size_bytes: session.bytes_written,
                file_url: None,
                download_url: None,
                quality: session.quality.clone(),
                started_at: session.started_at,
                ended_at: None,
                processed_at: None,
                expires_at: None,
                view_count: 0,
                download_count: 0,
            });
        }
        drop(sessions);

        // Query database
        self.get_recording_from_db(recording_id).await
    }

    pub async fn get_transcription(
        &self,
        transcription_id: Uuid,
    ) -> Result<WebinarTranscription, RecordingError> {
        // Check active jobs first
        let jobs = self.transcription_jobs.read().await;
        if let Some(job) = jobs.get(&transcription_id) {
            return Ok(WebinarTranscription {
                id: job.id,
                webinar_id: job.webinar_id,
                recording_id: job.recording_id,
                status: job.status.clone(),
                language: job.language.clone(),
                duration_seconds: 0,
                word_count: 0,
                speaker_count: 0,
                segments: vec![],
                full_text: None,
                vtt_url: None,
                srt_url: None,
                json_url: None,
                created_at: job.created_at,
                completed_at: job.completed_at,
                confidence_score: 0.0,
            });
        }
        drop(jobs);

        // Query database
        self.get_transcription_from_db(transcription_id).await
    }

    pub async fn export_transcription(
        &self,
        transcription_id: Uuid,
        request: ExportTranscriptionRequest,
    ) -> Result<ExportTranscriptionResponse, RecordingError> {
        let transcription = self.get_transcription(transcription_id).await?;

        if transcription.status != TranscriptionStatus::Completed {
            return Err(RecordingError::TranscriptionNotReady);
        }

        let (content, content_type, extension) = match request.format {
            TranscriptionFormat::PlainText => {
                let text = self.format_as_plain_text(&transcription, &request);
                (text, "text/plain", "txt")
            }
            TranscriptionFormat::Vtt => {
                let vtt = self.format_as_vtt(&transcription, &request);
                (vtt, "text/vtt", "vtt")
            }
            TranscriptionFormat::Srt => {
                let srt = self.format_as_srt(&transcription, &request);
                (srt, "application/x-subrip", "srt")
            }
            TranscriptionFormat::Json => {
                let json = serde_json::to_string_pretty(&transcription)
                    .map_err(|e| RecordingError::ExportFailed(e.to_string()))?;
                (json, "application/json", "json")
            }
        };

        Ok(ExportTranscriptionResponse {
            format: request.format,
            content,
            content_type: content_type.to_string(),
            filename: format!("transcription_{}.{}", transcription_id, extension),
        })
    }

    fn format_as_plain_text(
        &self,
        transcription: &WebinarTranscription,
        request: &ExportTranscriptionRequest,
    ) -> String {
        let mut output = String::new();

        for segment in &transcription.segments {
            if request.include_speaker_names {
                if let Some(ref speaker) = segment.speaker_name {
                    output.push_str(&format!("[{}] ", speaker));
                }
            }
            if request.include_timestamps {
                output.push_str(&format!(
                    "[{} - {}] ",
                    format_timestamp_plain(segment.start_time_ms as i64),
                    format_timestamp_plain(segment.end_time_ms as i64)
                ));
            }
            output.push_str(&segment.text);
            output.push('\n');
        }

        output
    }

    fn format_as_vtt(
        &self,
        transcription: &WebinarTranscription,
        request: &ExportTranscriptionRequest,
    ) -> String {
        let mut output = String::from("WEBVTT\n\n");

        for (i, segment) in transcription.segments.iter().enumerate() {
            output.push_str(&format!("{}\n", i + 1));
            output.push_str(&format!(
                "{} --> {}\n",
                format_timestamp_vtt(segment.start_time_ms as i64),
                format_timestamp_vtt(segment.end_time_ms as i64)
            ));

            if request.include_speaker_names {
                if let Some(ref speaker) = segment.speaker_name {
                    output.push_str(&format!("<v {}>{}</v>\n\n", speaker, segment.text));
                    continue;
                }
            }
            output.push_str(&format!("{}\n\n", segment.text));
        }

        output
    }

    fn format_as_srt(
        &self,
        transcription: &WebinarTranscription,
        request: &ExportTranscriptionRequest,
    ) -> String {
        let mut output = String::new();

        for (i, segment) in transcription.segments.iter().enumerate() {
            output.push_str(&format!("{}\n", i + 1));
            output.push_str(&format!(
                "{} --> {}\n",
                format_timestamp_srt(segment.start_time_ms as i64),
                format_timestamp_srt(segment.end_time_ms as i64)
            ));

            let mut text = segment.text.clone();
            if request.include_speaker_names {
                if let Some(ref speaker) = segment.speaker_name {
                    text = format!("[{}] {}", speaker, text);
                }
            }
            output.push_str(&format!("{}\n\n", text));
        }

        output
    }

    pub async fn list_recordings(
        &self,
        webinar_id: Uuid,
    ) -> Result<Vec<WebinarRecording>, RecordingError> {
        self.list_recordings_from_db(webinar_id).await
    }

    pub async fn delete_recording(&self, recording_id: Uuid) -> Result<(), RecordingError> {
        // Check if recording is active
        let sessions = self.active_sessions.read().await;
        if sessions.contains_key(&recording_id) {
            return Err(RecordingError::InvalidState(
                "Cannot delete active recording".to_string(),
            ));
        }
        drop(sessions);

        // Delete from storage
        self.delete_recording_files(recording_id).await?;

        // Delete from database
        self.delete_recording_from_db(recording_id).await
    }

    // Database helper methods

    async fn get_recording_from_db(&self, recording_id: Uuid) -> Result<WebinarRecording, RecordingError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            let row: (Uuid, Uuid, String, Option<String>, Option<i64>, Option<i32>, String, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>) = meeting_recordings::table
                .filter(meeting_recordings::id.eq(recording_id))
                .select((
                    meeting_recordings::id,
                    meeting_recordings::room_id,
                    meeting_recordings::recording_type,
                    meeting_recordings::file_url,
                    meeting_recordings::file_size,
                    meeting_recordings::duration_seconds,
                    meeting_recordings::status,
                    meeting_recordings::started_at,
                    meeting_recordings::stopped_at,
                    meeting_recordings::processed_at,
                ))
                .first(&mut conn)
                .map_err(|_| RecordingError::NotFound)?;

            let status = match row.6.as_str() {
                "recording" => RecordingStatus::Recording,
                "processing" => RecordingStatus::Processing,
                "ready" => RecordingStatus::Ready,
                "failed" => RecordingStatus::Failed,
                "deleted" => RecordingStatus::Deleted,
                _ => RecordingStatus::Failed,
            };

            let quality = match row.2.as_str() {
                "high" | "hd" => RecordingQuality::High,
                "low" | "audio" => RecordingQuality::AudioOnly,
                _ => RecordingQuality::Standard,
            };

            Ok(WebinarRecording {
                id: row.0,
                webinar_id: row.1,
                status,
                duration_seconds: row.5.unwrap_or(0) as u64,
                file_size_bytes: row.4.unwrap_or(0) as u64,
                file_url: row.3.clone(),
                download_url: row.3,
                quality,
                started_at: row.7,
                ended_at: row.8,
                processed_at: row.9,
                expires_at: None,
                view_count: 0,
                download_count: 0,
            })
        })
        .await
        .map_err(|e| RecordingError::DatabaseError(e.to_string()))?
    }

    async fn delete_recording_from_db(&self, recording_id: Uuid) -> Result<(), RecordingError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            diesel::update(meeting_recordings::table.filter(meeting_recordings::id.eq(recording_id)))
                .set((
                    meeting_recordings::status.eq("deleted"),
                    meeting_recordings::updated_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            Ok(())
        })
        .await
        .map_err(|e| RecordingError::DatabaseError(e.to_string()))?
    }

    async fn list_recordings_from_db(&self, room_id: Uuid) -> Result<Vec<WebinarRecording>, RecordingError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            let rows: Vec<(Uuid, Uuid, String, Option<String>, Option<i64>, Option<i32>, String, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>)> = meeting_recordings::table
                .filter(meeting_recordings::room_id.eq(room_id))
                .filter(meeting_recordings::status.ne("deleted"))
                .order(meeting_recordings::started_at.desc())
                .select((
                    meeting_recordings::id,
                    meeting_recordings::room_id,
                    meeting_recordings::recording_type,
                    meeting_recordings::file_url,
                    meeting_recordings::file_size,
                    meeting_recordings::duration_seconds,
                    meeting_recordings::status,
                    meeting_recordings::started_at,
                    meeting_recordings::stopped_at,
                    meeting_recordings::processed_at,
                ))
                .load(&mut conn)
                .map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            let recordings = rows.into_iter().map(|row| {
                let status = match row.6.as_str() {
                    "recording" => RecordingStatus::Recording,
                    "processing" => RecordingStatus::Processing,
                    "ready" => RecordingStatus::Ready,
                    "failed" => RecordingStatus::Failed,
                    "deleted" => RecordingStatus::Deleted,
                    _ => RecordingStatus::Failed,
                };

                let quality = match row.2.as_str() {
                    "high" | "hd" => RecordingQuality::High,
                    "low" | "audio" => RecordingQuality::AudioOnly,
                    _ => RecordingQuality::Standard,
                };

                WebinarRecording {
                    id: row.0,
                    webinar_id: row.1,
                    status,
                    duration_seconds: row.5.unwrap_or(0) as u64,
                    file_size_bytes: row.4.unwrap_or(0) as u64,
                    file_url: row.3.clone(),
                    download_url: row.3,
                    quality,
                    started_at: row.7,
                    ended_at: row.8,
                    processed_at: row.9,
                    expires_at: None,
                    view_count: 0,
                    download_count: 0,
                }
            }).collect();

            Ok(recordings)
        })
        .await
        .map_err(|e| RecordingError::DatabaseError(e.to_string()))?
    }

    async fn create_recording_record(
        &self,
        recording_id: Uuid,
        webinar_id: Uuid,
        quality: &RecordingQuality,
        started_at: DateTime<Utc>,
    ) -> Result<(), RecordingError> {
        let pool = self.pool.clone();
        let quality_str = match quality {
            RecordingQuality::AudioOnly => "audio_only",
            RecordingQuality::Standard => "standard",
            RecordingQuality::High => "high",
            RecordingQuality::Ultra => "ultra",
        }.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            // Get org_id and bot_id from room - for now use defaults
            let org_id = Uuid::nil();
            let bot_id = Uuid::nil();

            diesel::insert_into(meeting_recordings::table)
                .values((
                    meeting_recordings::id.eq(recording_id),
                    meeting_recordings::room_id.eq(webinar_id),
                    meeting_recordings::org_id.eq(org_id),
                    meeting_recordings::bot_id.eq(bot_id),
                    meeting_recordings::recording_type.eq(&quality_str),
                    meeting_recordings::status.eq("recording"),
                    meeting_recordings::started_at.eq(started_at),
                    meeting_recordings::metadata.eq(serde_json::json!({})),
                    meeting_recordings::created_at.eq(Utc::now()),
                    meeting_recordings::updated_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            Ok(())
        })
        .await
        .map_err(|e| RecordingError::DatabaseError(e.to_string()))?
    }

    async fn update_recording_stopped(
        &self,
        recording_id: Uuid,
        ended_at: DateTime<Utc>,
        duration_seconds: u64,
        file_size_bytes: u64,
    ) -> Result<(), RecordingError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            diesel::update(meeting_recordings::table.filter(meeting_recordings::id.eq(recording_id)))
                .set((
                    meeting_recordings::status.eq("processing"),
                    meeting_recordings::stopped_at.eq(ended_at),
                    meeting_recordings::duration_seconds.eq(duration_seconds as i32),
                    meeting_recordings::file_size.eq(file_size_bytes as i64),
                    meeting_recordings::updated_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            Ok(())
        })
        .await
        .map_err(|e| RecordingError::DatabaseError(e.to_string()))?
    }

    async fn update_recording_processed(
        &self,
        recording_id: Uuid,
        file_url: &str,
        _download_url: &str,
    ) -> Result<(), RecordingError> {
        let pool = self.pool.clone();
        let file_url = file_url.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            diesel::update(meeting_recordings::table.filter(meeting_recordings::id.eq(recording_id)))
                .set((
                    meeting_recordings::status.eq("ready"),
                    meeting_recordings::file_url.eq(&file_url),
                    meeting_recordings::processed_at.eq(Utc::now()),
                    meeting_recordings::updated_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .map_err(|e| RecordingError::DatabaseError(e.to_string()))?;

            Ok(())
        })
        .await
        .map_err(|e| RecordingError::DatabaseError(e.to_string()))?
    }

    async fn create_transcription_record(
        &self,
        _transcription_id: Uuid,
        _recording_id: Uuid,
        _language: &str,
    ) -> Result<(), RecordingError> {
        // Transcription records use a separate table - implement when needed
        Ok(())
    }

    pub fn clone_for_task(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            config: self.config.clone(),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            transcription_jobs: Arc::new(RwLock::new(HashMap::new())),
            event_sender: self.event_sender.clone(),
        }
    }

    async fn update_transcription_completed(
        &self,
        _transcription_id: Uuid,
        _text: &str,
        _segments: &[TranscriptionSegment],
    ) -> Result<(), RecordingError> {
        Ok(())
    }

    async fn get_transcription_from_db(
        &self,
        _transcription_id: Uuid,
    ) -> Result<WebinarTranscription, RecordingError> {
        Err(RecordingError::NotFound)
    }

    async fn delete_recording_files(&self, _recording_id: Uuid) -> Result<(), RecordingError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_status_display() {
        assert_eq!(format!("{:?}", RecordingStatus::Pending), "Pending");
        assert_eq!(format!("{:?}", RecordingStatus::Recording), "Recording");
        assert_eq!(format!("{:?}", RecordingStatus::Completed), "Completed");
    }

    #[test]
    fn test_recording_format_display() {
        assert_eq!(format!("{:?}", RecordingFormat::WebM), "WebM");
        assert_eq!(format!("{:?}", RecordingFormat::Mp4), "Mp4");
    }
}
