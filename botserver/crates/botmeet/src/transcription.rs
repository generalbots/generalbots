use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub id: String,
    pub meeting_id: String,
    pub speaker: String,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f32,
    pub is_final: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerInfo {
    pub speaker_id: String,
    pub speaker_name: String,
    pub segments: Vec<usize>,
    pub total_duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTranscript {
    pub meeting_id: String,
    pub segments: Vec<TranscriptionSegment>,
    pub speakers: Vec<SpeakerInfo>,
    pub generated_at: DateTime<Utc>,
    pub total_duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSummary {
    pub meeting_id: String,
    pub participant_count: usize,
    pub total_segments: usize,
    pub duration_seconds: f64,
    pub key_topics: Vec<String>,
    pub speaker_talk_ratio: Vec<SpeakerRatio>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerRatio {
    pub speaker: String,
    pub segment_count: usize,
    pub word_count: usize,
    pub percentage: f32,
}

#[async_trait]
pub trait AudioTranscriber: Send + Sync {
    async fn transcribe(&self, audio_data: &[u8]) -> Result<Vec<TranscriptionSegment>>;
    async fn transcribe_stream(&self, audio_chunk: &[u8], meeting_id: &str) -> Result<Option<TranscriptionSegment>>;
}

pub struct SttTranscriber;

#[async_trait]
impl AudioTranscriber for SttTranscriber {
    async fn transcribe(&self, audio_data: &[u8]) -> Result<Vec<TranscriptionSegment>> {
        info!("STT transcribe called with {} bytes", audio_data.len());

        let text = String::from_utf8_lossy(audio_data)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        if text.is_empty() {
            return Ok(vec![TranscriptionSegment {
                id: Uuid::new_v4().to_string(),
                meeting_id: String::new(),
                speaker: "unknown".to_string(),
                text: "[Audio transcription would appear here]".to_string(),
                start_time: 0.0,
                end_time: 1.0,
                confidence: 0.85,
                is_final: true,
                created_at: Utc::now(),
            }]);
        }

        Ok(vec![TranscriptionSegment {
            id: Uuid::new_v4().to_string(),
            meeting_id: String::new(),
            speaker: "unknown".to_string(),
            text: text.clone(),
            start_time: 0.0,
            end_time: text.len() as f64 * 0.05,
            confidence: 0.92,
            is_final: true,
            created_at: Utc::now(),
        }])
    }

    async fn transcribe_stream(
        &self,
        audio_chunk: &[u8],
        meeting_id: &str,
    ) -> Result<Option<TranscriptionSegment>> {
        info!("Stream transcribe: {} bytes for meeting {}", audio_chunk.len(), meeting_id);

        if audio_chunk.len() < 160 {
            return Ok(None);
        }

        Ok(Some(TranscriptionSegment {
            id: Uuid::new_v4().to_string(),
            meeting_id: meeting_id.to_string(),
            speaker: "unknown".to_string(),
            text: "[Live transcription chunk]".to_string(),
            start_time: 0.0,
            end_time: audio_chunk.len() as f64 * 0.001,
            confidence: 0.75,
            is_final: false,
            created_at: Utc::now(),
        }))
    }
}

pub struct TranscriptionService {
    transcriber: Box<dyn AudioTranscriber + Send + Sync>,
    segments: Vec<TranscriptionSegment>,
}

impl TranscriptionService {
    pub fn new() -> Self {
        Self {
            transcriber: Box::new(SttTranscriber),
            segments: Vec::new(),
        }
    }

    pub fn with_transcriber(transcriber: Box<dyn AudioTranscriber + Send + Sync>) -> Self {
        Self {
            transcriber,
            segments: Vec::new(),
        }
    }

    pub async fn process_audio(&mut self, audio_data: &[u8], meeting_id: &str) -> Result<Vec<TranscriptionSegment>> {
        let mut segments = self.transcriber.transcribe(audio_data).await?;
        for seg in &mut segments {
            seg.meeting_id = meeting_id.to_string();
        }
        let count = segments.len();
        self.segments.extend(segments.clone());
        info!("Processed {} transcription segments for meeting {}", count, meeting_id);
        Ok(segments)
    }

    pub async fn process_audio_chunk(&mut self, chunk: &[u8], meeting_id: &str) -> Result<Option<TranscriptionSegment>> {
        let segment = self.transcriber.transcribe_stream(chunk, meeting_id).await?;
        if let Some(ref seg) = segment {
            self.segments.push(seg.clone());
        }
        Ok(segment)
    }

    pub fn speaker_diarization(&mut self) -> Vec<SpeakerInfo> {
        let mut speakers: Vec<SpeakerInfo> = Vec::new();

        for (i, seg) in self.segments.iter().enumerate() {
            let existing = speakers.iter_mut().find(|s: &&mut SpeakerInfo| s.speaker_id == seg.speaker);
            if let Some(speaker) = existing {
                speaker.segments.push(i);
                speaker.total_duration += seg.end_time - seg.start_time;
            } else {
                speakers.push(SpeakerInfo {
                    speaker_id: seg.speaker.clone(),
                    speaker_name: format!("Speaker {}", speakers.len() + 1),
                    segments: vec![i],
                    total_duration: seg.end_time - seg.start_time,
                });
            }
        }

        speakers
    }

    pub fn generate_full_transcript(&self) -> FullTranscript {
        let meeting_id = self.segments.first().map(|s| s.meeting_id.clone()).unwrap_or_default();
        let speakers = self.speaker_diarization();
        let total_duration = speakers.iter().map(|s| s.total_duration).sum();

        FullTranscript {
            meeting_id: meeting_id.clone(),
            segments: self.segments.clone(),
            speakers,
            generated_at: Utc::now(),
            total_duration_seconds: total_duration,
        }
    }

    pub fn generate_transcript_summary(&self) -> TranscriptSummary {
        let speakers = self.speaker_diarization();
        let total_segments = self.segments.len();
        let total_duration: f64 = speakers.iter().map(|s| s.total_duration).sum();

        let total_words: usize = self.segments.iter().map(|s| s.text.split_whitespace().count()).sum();
        let speaker_ratios: Vec<SpeakerRatio> = speakers
            .iter()
            .map(|s| {
                let word_count: usize = s
                    .segments
                    .iter()
                    .map(|&idx| self.segments[idx].text.split_whitespace().count())
                    .sum();
                let percentage = if total_words > 0 {
                    (word_count as f32 / total_words as f32) * 100.0
                } else {
                    0.0
                };
                SpeakerRatio {
                    speaker: s.speaker_name.clone(),
                    segment_count: s.segments.len(),
                    word_count,
                    percentage,
                }
            })
            .collect();

        let meeting_id = self.segments.first().map(|s| s.meeting_id.clone()).unwrap_or_default();

        TranscriptSummary {
            meeting_id,
            participant_count: speakers.len(),
            total_segments,
            duration_seconds: total_duration,
            key_topics: Vec::new(),
            speaker_talk_ratio: speaker_ratios,
            generated_at: Utc::now(),
        }
    }

    pub fn get_segments(&self) -> &[TranscriptionSegment] {
        &self.segments
    }

    pub fn clear_segments(&mut self) {
        self.segments.clear();
    }

    pub fn assign_speaker_names(&mut self, mapping: &[(String, String)]) {
        for seg in &mut self.segments {
            for (speaker_id, name) in mapping {
                if seg.speaker == *speaker_id {
                    seg.speaker = name.clone();
                    break;
                }
            }
        }
    }
}
