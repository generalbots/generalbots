use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};
use uuid::Uuid;

use crate::security::command_guard::SafeCommand;
use crate::core::shared::utils::DbPool;

use super::models::*;
use super::schema::*;

pub struct VideoEngine {
    pub db: DbPool,
    pub cache: Option<Arc<redis::Client>>,
    pub progress_tx: Option<broadcast::Sender<ExportProgressEvent>>,
}

impl VideoEngine {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            cache: None,
            progress_tx: None,
        }
    }

    pub fn with_cache(db: DbPool, cache: Arc<redis::Client>) -> Self {
        Self {
            db,
            cache: Some(cache),
            progress_tx: None,
        }
    }

    fn get_conn(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        diesel::result::Error,
    > {
        self.db.get().map_err(|e| {
            error!("DB connection error: {e}");
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })
    }

    pub async fn create_project(
        &self,
        organization_id: Option<Uuid>,
        created_by: Option<Uuid>,
        req: CreateProjectRequest,
    ) -> Result<VideoProject, diesel::result::Error> {
        let mut conn = self.get_conn()?;

        let project = VideoProject {
            id: Uuid::new_v4(),
            organization_id,
            created_by,
            name: req.name,
            description: req.description,
            resolution_width: req.resolution_width.unwrap_or(1920),
            resolution_height: req.resolution_height.unwrap_or(1080),
            fps: req.fps.unwrap_or(30),
            total_duration_ms: 0,
            timeline_json: serde_json::json!({"clips": []}),
            layers_json: serde_json::json!([]),
            audio_tracks_json: serde_json::json!([]),
            playhead_ms: 0,
            selection_json: serde_json::json!({"type": "None"}),
            zoom_level: 1.0,
            thumbnail_url: None,
            status: ProjectStatus::Draft.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        diesel::insert_into(video_projects::table)
            .values(&project)
            .execute(&mut conn)?;

        info!("Created video project: {} ({})", project.name, project.id);
        Ok(project)
    }

    pub async fn get_project(&self, project_id: Uuid) -> Result<VideoProject, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        video_projects::table.find(project_id).first(&mut conn)
    }

    pub async fn list_projects(
        &self,
        org_id: Option<Uuid>,
        filters: ProjectFilters,
    ) -> Result<Vec<ProjectResponse>, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        let mut query = video_projects::table.into_boxed();

        if let Some(org) = org_id {
            query = query.filter(video_projects::organization_id.eq(org));
        }
        if let Some(status) = filters.status {
            query = query.filter(video_projects::status.eq(status));
        }
        if let Some(search) = filters.search {
            let pattern = format!("%{search}%");
            query = query.filter(video_projects::name.ilike(pattern));
        }

        query = query.order(video_projects::updated_at.desc());

        if let Some(limit) = filters.limit {
            query = query.limit(limit);
        }
        if let Some(offset) = filters.offset {
            query = query.offset(offset);
        }

        let projects: Vec<VideoProject> = query.load(&mut conn)?;

        let responses: Vec<ProjectResponse> = projects
            .into_iter()
            .map(|p| ProjectResponse {
                id: p.id,
                name: p.name,
                description: p.description,
                resolution_width: p.resolution_width,
                resolution_height: p.resolution_height,
                fps: p.fps,
                total_duration_ms: p.total_duration_ms,
                playhead_ms: p.playhead_ms,
                zoom_level: p.zoom_level,
                thumbnail_url: p.thumbnail_url,
                status: p.status,
                clips_count: 0,
                layers_count: 0,
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect();

        Ok(responses)
    }

    pub async fn update_project(
        &self,
        project_id: Uuid,
        req: UpdateProjectRequest,
    ) -> Result<VideoProject, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        let project: VideoProject = video_projects::table.find(project_id).first(&mut conn)?;

        diesel::update(video_projects::table.find(project_id))
            .set((
                video_projects::name.eq(req.name.unwrap_or(project.name)),
                video_projects::description.eq(req.description.or(project.description)),
                video_projects::playhead_ms.eq(req.playhead_ms.unwrap_or(project.playhead_ms)),
                video_projects::selection_json.eq(req.selection_json.unwrap_or(project.selection_json)),
                video_projects::zoom_level.eq(req.zoom_level.unwrap_or(project.zoom_level)),
                video_projects::status.eq(req.status.unwrap_or(project.status)),
                video_projects::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        self.get_project(project_id).await
    }

    pub async fn delete_project(&self, project_id: Uuid) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_conn()?;

        diesel::delete(video_clips::table.filter(video_clips::project_id.eq(project_id)))
            .execute(&mut conn)?;
        diesel::delete(video_layers::table.filter(video_layers::project_id.eq(project_id)))
            .execute(&mut conn)?;
        diesel::delete(video_audio_tracks::table.filter(video_audio_tracks::project_id.eq(project_id)))
            .execute(&mut conn)?;
        diesel::delete(video_exports::table.filter(video_exports::project_id.eq(project_id)))
            .execute(&mut conn)?;
        diesel::delete(video_command_history::table.filter(video_command_history::project_id.eq(project_id)))
            .execute(&mut conn)?;
        diesel::delete(video_analytics::table.filter(video_analytics::project_id.eq(project_id)))
            .execute(&mut conn)?;
        diesel::delete(video_projects::table.find(project_id)).execute(&mut conn)?;

        info!("Deleted video project: {project_id}");
        Ok(())
    }

    pub async fn get_project_detail(
        &self,
        project_id: Uuid,
    ) -> Result<ProjectDetailResponse, diesel::result::Error> {
        let project = self.get_project(project_id).await?;
        let clips = self.get_clips(project_id).await?;
        let layers = self.get_layers(project_id).await?;
        let audio_tracks = self.get_audio_tracks(project_id).await?;

        Ok(ProjectDetailResponse {
            project: ProjectResponse {
                id: project.id,
                name: project.name,
                description: project.description,
                resolution_width: project.resolution_width,
                resolution_height: project.resolution_height,
                fps: project.fps,
                total_duration_ms: project.total_duration_ms,
                playhead_ms: project.playhead_ms,
                zoom_level: project.zoom_level,
                thumbnail_url: project.thumbnail_url,
                status: project.status,
                clips_count: clips.len(),
                layers_count: layers.len(),
                created_at: project.created_at,
                updated_at: project.updated_at,
            },
            clips,
            layers,
            audio_tracks,
        })
    }

    pub async fn add_clip(
        &self,
        project_id: Uuid,
        req: AddClipRequest,
    ) -> Result<VideoClip, diesel::result::Error> {
        let mut conn = self.get_conn()?;

        let max_order: Option<i32> = video_clips::table
            .filter(video_clips::project_id.eq(project_id))
            .select(diesel::dsl::max(video_clips::clip_order))
            .first(&mut conn)?;

        let clip = VideoClip {
            id: Uuid::new_v4(),
            project_id,
            name: req.name.unwrap_or_else(|| "Clip".to_string()),
            source_url: req.source_url,
            start_ms: req.at_ms.unwrap_or(0),
            duration_ms: req.duration_ms.unwrap_or(5000),
            trim_in_ms: 0,
            trim_out_ms: 0,
            volume: 1.0,
            clip_order: max_order.unwrap_or(0) + 1,
            transition_in: None,
            transition_out: None,
            created_at: Utc::now(),
        };

        diesel::insert_into(video_clips::table)
            .values(&clip)
            .execute(&mut conn)?;

        self.recalculate_duration(project_id).await?;
        Ok(clip)
    }

    pub async fn get_clips(&self, project_id: Uuid) -> Result<Vec<VideoClip>, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        video_clips::table
            .filter(video_clips::project_id.eq(project_id))
            .order(video_clips::clip_order.asc())
            .load(&mut conn)
    }

    pub async fn update_clip(
        &self,
        clip_id: Uuid,
        req: UpdateClipRequest,
    ) -> Result<VideoClip, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        let clip: VideoClip = video_clips::table.find(clip_id).first(&mut conn)?;

        diesel::update(video_clips::table.find(clip_id))
            .set((
                video_clips::name.eq(req.name.unwrap_or(clip.name)),
                video_clips::start_ms.eq(req.start_ms.unwrap_or(clip.start_ms)),
                video_clips::duration_ms.eq(req.duration_ms.unwrap_or(clip.duration_ms)),
                video_clips::trim_in_ms.eq(req.trim_in_ms.unwrap_or(clip.trim_in_ms)),
                video_clips::trim_out_ms.eq(req.trim_out_ms.unwrap_or(clip.trim_out_ms)),
                video_clips::volume.eq(req.volume.unwrap_or(clip.volume)),
                video_clips::transition_in.eq(req.transition_in.or(clip.transition_in)),
                video_clips::transition_out.eq(req.transition_out.or(clip.transition_out)),
            ))
            .execute(&mut conn)?;

        self.recalculate_duration(clip.project_id).await?;
        video_clips::table.find(clip_id).first(&mut conn)
    }

    pub async fn delete_clip(&self, clip_id: Uuid) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_conn()?;
        let clip: VideoClip = video_clips::table.find(clip_id).first(&mut conn)?;
        let project_id = clip.project_id;

        diesel::delete(video_clips::table.find(clip_id)).execute(&mut conn)?;
        self.recalculate_duration(project_id).await?;
        Ok(())
    }

    pub async fn split_clip(
        &self,
        clip_id: Uuid,
        at_ms: i64,
    ) -> Result<(VideoClip, VideoClip), diesel::result::Error> {
        let mut conn = self.get_conn()?;
        let clip: VideoClip = video_clips::table.find(clip_id).first(&mut conn)?;

        let split_point = at_ms - clip.start_ms;
        if split_point <= 0 || split_point >= clip.duration_ms {
            return Err(diesel::result::Error::NotFound);
        }

        let first_duration = split_point;
        let second_duration = clip.duration_ms - split_point;

        diesel::update(video_clips::table.find(clip_id))
            .set((
                video_clips::duration_ms.eq(first_duration),
                video_clips::trim_out_ms.eq(clip.trim_out_ms + second_duration),
                video_clips::transition_out.eq(None::<String>),
            ))
            .execute(&mut conn)?;

        let second_clip = VideoClip {
            id: Uuid::new_v4(),
            project_id: clip.project_id,
            name: format!("{} (split)", clip.name),
            source_url: clip.source_url.clone(),
            start_ms: clip.start_ms + first_duration,
            duration_ms: second_duration,
            trim_in_ms: clip.trim_in_ms + first_duration,
            trim_out_ms: clip.trim_out_ms,
            volume: clip.volume,
            clip_order: clip.clip_order + 1,
            transition_in: None,
            transition_out: clip.transition_out,
            created_at: Utc::now(),
        };

        diesel::insert_into(video_clips::table)
            .values(&second_clip)
            .execute(&mut conn)?;

        let first_clip: VideoClip = video_clips::table.find(clip_id).first(&mut conn)?;
        self.recalculate_duration(clip.project_id).await?;

        info!("Split clip {} at {}ms", clip_id, at_ms);
        Ok((first_clip, second_clip))
    }

    pub async fn add_layer(
        &self,
        project_id: Uuid,
        req: AddLayerRequest,
    ) -> Result<VideoLayer, diesel::result::Error> {
        let mut conn = self.get_conn()?;

        let max_track: Option<i32> = video_layers::table
            .filter(video_layers::project_id.eq(project_id))
            .select(diesel::dsl::max(video_layers::track_index))
            .first(&mut conn)?;

        let layer = VideoLayer {
            id: Uuid::new_v4(),
            project_id,
            name: req.name.unwrap_or_else(|| format!("{} Layer", req.layer_type)),
            layer_type: req.layer_type,
            track_index: max_track.unwrap_or(0) + 1,
            start_ms: req.start_ms.unwrap_or(0),
            end_ms: req.end_ms.unwrap_or(5000),
            x: req.x.unwrap_or(0.5),
            y: req.y.unwrap_or(0.5),
            width: req.width.unwrap_or(0.5),
            height: req.height.unwrap_or(0.2),
            rotation: 0.0,
            opacity: 1.0,
            properties_json: req.properties.unwrap_or(serde_json::json!({})),
            animation_in: None,
            animation_out: None,
            locked: false,
            keyframes_json: None,
            created_at: Utc::now(),
        };

        diesel::insert_into(video_layers::table)
            .values(&layer)
            .execute(&mut conn)?;

        Ok(layer)
    }

    pub async fn get_layers(&self, project_id: Uuid) -> Result<Vec<VideoLayer>, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        video_layers::table
            .filter(video_layers::project_id.eq(project_id))
            .order(video_layers::track_index.asc())
            .load(&mut conn)
    }

    pub async fn update_layer(
        &self,
        layer_id: Uuid,
        req: UpdateLayerRequest,
    ) -> Result<VideoLayer, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        let layer: VideoLayer = video_layers::table.find(layer_id).first(&mut conn)?;

        diesel::update(video_layers::table.find(layer_id))
            .set((
                video_layers::name.eq(req.name.unwrap_or(layer.name)),
                video_layers::start_ms.eq(req.start_ms.unwrap_or(layer.start_ms)),
                video_layers::end_ms.eq(req.end_ms.unwrap_or(layer.end_ms)),
                video_layers::x.eq(req.x.unwrap_or(layer.x)),
                video_layers::y.eq(req.y.unwrap_or(layer.y)),
                video_layers::width.eq(req.width.unwrap_or(layer.width)),
                video_layers::height.eq(req.height.unwrap_or(layer.height)),
                video_layers::rotation.eq(req.rotation.unwrap_or(layer.rotation)),
                video_layers::opacity.eq(req.opacity.unwrap_or(layer.opacity)),
                video_layers::properties_json.eq(req.properties.unwrap_or(layer.properties_json)),
                video_layers::animation_in.eq(req.animation_in.or(layer.animation_in)),
                video_layers::animation_out.eq(req.animation_out.or(layer.animation_out)),
                video_layers::locked.eq(req.locked.unwrap_or(layer.locked)),
            ))
            .execute(&mut conn)?;

        video_layers::table.find(layer_id).first(&mut conn)
    }

    pub async fn delete_layer(&self, layer_id: Uuid) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_conn()?;
        diesel::delete(video_keyframes::table.filter(video_keyframes::layer_id.eq(layer_id)))
            .execute(&mut conn)?;
        diesel::delete(video_layers::table.find(layer_id)).execute(&mut conn)?;
        Ok(())
    }

    pub async fn add_audio_track(
        &self,
        project_id: Uuid,
        req: AddAudioRequest,
    ) -> Result<VideoAudioTrack, diesel::result::Error> {
        let mut conn = self.get_conn()?;

        let track = VideoAudioTrack {
            id: Uuid::new_v4(),
            project_id,
            name: req.name.unwrap_or_else(|| "Audio Track".to_string()),
            source_url: req.source_url,
            track_type: req.track_type.unwrap_or_else(|| "music".to_string()),
            start_ms: req.start_ms.unwrap_or(0),
            duration_ms: req.duration_ms.unwrap_or(10000),
            volume: req.volume.unwrap_or(1.0),
            fade_in_ms: 0,
            fade_out_ms: 0,
            waveform_json: None,
            beat_markers_json: None,
            created_at: Utc::now(),
        };

        diesel::insert_into(video_audio_tracks::table)
            .values(&track)
            .execute(&mut conn)?;

        Ok(track)
    }

    pub async fn get_audio_tracks(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<VideoAudioTrack>, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        video_audio_tracks::table
            .filter(video_audio_tracks::project_id.eq(project_id))
            .order(video_audio_tracks::start_ms.asc())
            .load(&mut conn)
    }

    pub async fn delete_audio_track(&self, track_id: Uuid) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_conn()?;
        diesel::delete(video_audio_tracks::table.find(track_id)).execute(&mut conn)?;
        Ok(())
    }

    pub async fn add_keyframe(
        &self,
        layer_id: Uuid,
        req: AddKeyframeRequest,
    ) -> Result<VideoKeyframe, diesel::result::Error> {
        let mut conn = self.get_conn()?;

        let keyframe = VideoKeyframe {
            id: Uuid::new_v4(),
            layer_id,
            property_name: req.property_name,
            time_ms: req.time_ms,
            value_json: req.value,
            easing: req.easing.unwrap_or_else(|| "linear".to_string()),
            created_at: Utc::now(),
        };

        diesel::insert_into(video_keyframes::table)
            .values(&keyframe)
            .execute(&mut conn)?;

        Ok(keyframe)
    }

    pub async fn get_keyframes(
        &self,
        layer_id: Uuid,
    ) -> Result<Vec<VideoKeyframe>, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        video_keyframes::table
            .filter(video_keyframes::layer_id.eq(layer_id))
            .order(video_keyframes::time_ms.asc())
            .load(&mut conn)
    }

    pub async fn delete_keyframe(&self, keyframe_id: Uuid) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_conn()?;
        diesel::delete(video_keyframes::table.find(keyframe_id)).execute(&mut conn)?;
        Ok(())
    }

    pub async fn generate_preview_frame(
        &self,
        project_id: Uuid,
        at_ms: i64,
        width: i32,
        height: i32,
        output_dir: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let clips = self.get_clips(project_id).await?;

        let clip = clips
            .iter()
            .find(|c| at_ms >= c.start_ms && at_ms < c.start_ms + c.duration_ms)
            .ok_or("No clip at this timestamp")?;

        let seek_time = (at_ms - clip.start_ms + clip.trim_in_ms) as f64 / 1000.0;
        let output_filename = format!("preview_{}_{}.jpg", project_id, at_ms);
        let output_path = format!("{}/{}", output_dir, output_filename);

        let cmd = SafeCommand::new("ffmpeg")
            .map_err(|e| format!("Command creation failed: {e}"))?
            .arg("-y").map_err(|e| format!("Arg error: {e}"))?
            .arg("-ss").map_err(|e| format!("Arg error: {e}"))?
            .arg(&format!("{:.3}", seek_time)).map_err(|e| format!("Arg error: {e}"))?
            .arg("-i").map_err(|e| format!("Arg error: {e}"))?
            .arg(&clip.source_url).map_err(|e| format!("Arg error: {e}"))?
            .arg("-vframes").map_err(|e| format!("Arg error: {e}"))?
            .arg("1").map_err(|e| format!("Arg error: {e}"))?
            .arg("-vf").map_err(|e| format!("Arg error: {e}"))?
            .arg(&format!("scale={}:{}", width, height)).map_err(|e| format!("Arg error: {e}"))?
            .arg("-q:v").map_err(|e| format!("Arg error: {e}"))?
            .arg("2").map_err(|e| format!("Arg error: {e}"))?
            .arg(&output_path).map_err(|e| format!("Arg error: {e}"))?;

        let result = cmd.execute().map_err(|e| format!("Execution failed: {e}"))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("FFmpeg error: {stderr}").into());
        }

        Ok(format!("/video/previews/{}", output_filename))
    }

    pub async fn transcribe_audio(
        &self,
        project_id: Uuid,
        clip_id: Option<Uuid>,
        language: Option<String>,
    ) -> Result<TranscriptionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let clips = self.get_clips(project_id).await?;

        let clip = if let Some(id) = clip_id {
            clips.iter().find(|c| c.id == id).ok_or("Clip not found")?
        } else {
            clips.first().ok_or("No clips in project")?
        };

        let botmodels_url =
            std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/audio/transcribe", botmodels_url))
            .json(&serde_json::json!({
                "audio_url": &clip.source_url,
                "language": language.unwrap_or_else(|| "auto".to_string()),
                "word_timestamps": true,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Transcription service error: {error}").into());
        }

        let result: serde_json::Value = response.json().await?;

        let segments: Vec<TranscriptionSegment> = result["segments"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|s| TranscriptionSegment {
                start_ms: (s["start"].as_f64().unwrap_or(0.0) * 1000.0) as i64,
                end_ms: (s["end"].as_f64().unwrap_or(0.0) * 1000.0) as i64,
                text: s["text"].as_str().unwrap_or("").to_string(),
                confidence: s["confidence"].as_f64().unwrap_or(0.9) as f32,
            })
            .collect();

        let full_text = segments
            .iter()
            .map(|s| s.text.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let duration_ms = segments.last().map(|s| s.end_ms).unwrap_or(0);

        Ok(TranscriptionResponse {
            segments,
            full_text,
            language: result["language"].as_str().unwrap_or("en").to_string(),
            duration_ms,
        })
    }

    pub async fn generate_captions_from_transcription(
        &self,
        project_id: Uuid,
        transcription: &TranscriptionResponse,
        _style: &str,
        _max_chars: i32,
        font_size: i32,
        color: &str,
        with_background: bool,
    ) -> Result<Vec<VideoLayer>, diesel::result::Error> {
        let mut layers = Vec::new();
        let bg_color = if with_background {
            Some("rgba(0,0,0,0.7)")
        } else {
            None
        };

        for segment in &transcription.segments {
            let layer = self
                .add_layer(
                    project_id,
                    AddLayerRequest {
                        name: Some(format!("Caption {}", layers.len() + 1)),
                        layer_type: "text".to_string(),
                        start_ms: Some(segment.start_ms),
                        end_ms: Some(segment.end_ms),
                        x: Some(0.5),
                        y: Some(0.85),
                        width: Some(0.9),
                        height: Some(0.1),
                        properties: Some(serde_json::json!({
                            "content": segment.text.trim(),
                            "font_family": "Arial",
                            "font_size": font_size,
                            "color": color,
                            "text_align": "center",
                            "background_color": bg_color,
                        })),
                    },
                )
                .await?;

            layers.push(layer);
        }

        info!("Generated {} caption layers for project {}", layers.len(), project_id);
        Ok(layers)
    }

    pub async fn text_to_speech(
        &self,
        text: &str,
        voice: &str,
        speed: f32,
        language: &str,
        output_dir: &str,
    ) -> Result<TTSResponse, Box<dyn std::error::Error + Send + Sync>> {
        let botmodels_url =
            std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/audio/tts", botmodels_url))
            .json(&serde_json::json!({
                "text": text,
                "voice": voice,
                "speed": speed,
                "language": language,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("TTS service error: {error}").into());
        }

        let audio_data = response.bytes().await?;
        let filename = format!("tts_{}.mp3", Uuid::new_v4());
        let file_path = format!("{}/{}", output_dir, filename);

        std::fs::write(&file_path, &audio_data)?;

        let duration_ms = self.get_audio_duration(&file_path).unwrap_or(0);

        Ok(TTSResponse {
            audio_url: format!("/video/audio/{}", filename),
            duration_ms,
        })
    }

    fn get_audio_duration(&self, path: &str) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let cmd = SafeCommand::new("ffprobe")
            .map_err(|e| format!("Command creation failed: {e}"))?
            .arg("-v").map_err(|e| format!("Arg error: {e}"))?
            .arg("error").map_err(|e| format!("Arg error: {e}"))?
            .arg("-show_entries").map_err(|e| format!("Arg error: {e}"))?
            .arg("format=duration").map_err(|e| format!("Arg error: {e}"))?
            .arg("-of").map_err(|e| format!("Arg error: {e}"))?
            .arg("default=noprint_wrappers=1:nokey=1").map_err(|e| format!("Arg error: {e}"))?
            .arg(path).map_err(|e| format!("Arg error: {e}"))?;

        let result = cmd.execute().map_err(|e| format!("Execution failed: {e}"))?;

        let stdout = String::from_utf8_lossy(&result.stdout);
        let duration_secs: f64 = stdout.trim().parse().unwrap_or(0.0);
        Ok((duration_secs * 1000.0) as i64)
    }

    pub async fn detect_scenes(
        &self,
        project_id: Uuid,
        threshold: f32,
        output_dir: &str,
    ) -> Result<SceneDetectionResponse, Box<dyn std::error::Error + Send + Sync>> {
        info!("Detecting scenes for project {project_id} with threshold {threshold}, output_dir: {output_dir}");
        let clips = self.get_clips(project_id).await?;
        let clip = clips.first().ok_or("No clips in project")?;

        let cmd = SafeCommand::new("ffmpeg")
            .map_err(|e| format!("Command creation failed: {e}"))?
            .arg("-i").map_err(|e| format!("Arg error: {e}"))?
            .arg(&clip.source_url).map_err(|e| format!("Arg error: {e}"))?
            .arg("-vf").map_err(|e| format!("Arg error: {e}"))?
            .arg(&format!("select='gt(scene,{})',showinfo", threshold)).map_err(|e| format!("Arg error: {e}"))?
            .arg("-f").map_err(|e| format!("Arg error: {e}"))?
            .arg("null").map_err(|e| format!("Arg error: {e}"))?
            .arg("-").map_err(|e| format!("Arg error: {e}"))?;

        let result = cmd.execute().map_err(|e| format!("Execution failed: {e}"))?;

        let mut scenes = Vec::new();
        let mut last_time: f64 = 0.0;

        let stderr = String::from_utf8_lossy(&result.stderr);
        for line in stderr.lines() {
            if line.contains("pts_time:") {
                if let Some(time_str) = line.split("pts_time:").nth(1) {
                    if let Some(time_end) = time_str.find(char::is_whitespace) {
                        if let Ok(time) = time_str[..time_end].parse::<f64>() {
                            if time > last_time + 0.5 {
                                scenes.push(SceneInfo {
                                    start_ms: (last_time * 1000.0) as i64,
                                    end_ms: (time * 1000.0) as i64,
                                    thumbnail_url: None,
                                    description: None,
                                });
                                last_time = time;
                            }
                        }
                    }
                }
            }
        }

        let duration_ms = self.get_audio_duration(&clip.source_url).unwrap_or(0);
        if last_time > 0.0 {
            scenes.push(SceneInfo {
                start_ms: (last_time * 1000.0) as i64,
                end_ms: duration_ms,
                thumbnail_url: None,
                description: None,
            });
        }

        info!("Detected {} scenes in project {}", scenes.len(), project_id);
        Ok(SceneDetectionResponse { scenes })
    }

    pub async fn auto_reframe(
        &self,
        project_id: Uuid,
        clip_id: Uuid,
        target_width: i32,
        target_height: i32,
        output_dir: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let clips = self.get_clips(project_id).await?;
        let clip = clips.iter().find(|c| c.id == clip_id).ok_or("Clip not found")?;

        let output_filename = format!("reframed_{}_{}.mp4", clip_id, target_width);
        let output_path = format!("{}/{}", output_dir, output_filename);

        let cmd = SafeCommand::new("ffmpeg")
            .map_err(|e| format!("Command creation failed: {e}"))?
            .arg("-i").map_err(|e| format!("Arg error: {e}"))?
            .arg(&clip.source_url).map_err(|e| format!("Arg error: {e}"))?
            .arg("-vf").map_err(|e| format!("Arg error: {e}"))?
            .arg(&format!(
                "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2",
                target_width, target_height, target_width, target_height
            )).map_err(|e| format!("Arg error: {e}"))?
            .arg("-c:a").map_err(|e| format!("Arg error: {e}"))?
            .arg("copy").map_err(|e| format!("Arg error: {e}"))?
            .arg(&output_path).map_err(|e| format!("Arg error: {e}"))?;

        let result = cmd.execute().map_err(|e| format!("Execution failed: {e}"))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Auto-reframe failed: {stderr}").into());
        }

        Ok(format!("/video/reframed/{}", output_filename))
    }

    pub fn get_available_templates(&self) -> Vec<TemplateInfo> {
        vec![
            TemplateInfo {
                id: "social-promo".to_string(),
                name: "Social Media Promo".to_string(),
                description: "Quick promo video with text animations".to_string(),
                thumbnail_url: "/templates/social-promo.jpg".to_string(),
                duration_ms: 15000,
                category: "social".to_string(),
            },
            TemplateInfo {
                id: "youtube-intro".to_string(),
                name: "YouTube Intro".to_string(),
                description: "Professional channel intro".to_string(),
                thumbnail_url: "/templates/youtube-intro.jpg".to_string(),
                duration_ms: 5000,
                category: "intro".to_string(),
            },
            TemplateInfo {
                id: "product-showcase".to_string(),
                name: "Product Showcase".to_string(),
                description: "Clean product presentation".to_string(),
                thumbnail_url: "/templates/product-showcase.jpg".to_string(),
                duration_ms: 30000,
                category: "business".to_string(),
            },
            TemplateInfo {
                id: "talking-head".to_string(),
                name: "Talking Head".to_string(),
                description: "Speaker with lower thirds".to_string(),
                thumbnail_url: "/templates/talking-head.jpg".to_string(),
                duration_ms: 60000,
                category: "presentation".to_string(),
            },
        ]
    }

    pub async fn apply_template(
        &self,
        project_id: Uuid,
        template_id: &str,
        customizations: Option<serde_json::Value>,
    ) -> Result<(), diesel::result::Error> {
        let custom = customizations.unwrap_or(serde_json::json!({}));
        let title = custom.get("title").and_then(|v| v.as_str()).unwrap_or("Title");

        match template_id {
            "social-promo" | "youtube-intro" => {
                self.add_layer(project_id, AddLayerRequest {
                    name: Some("Title".to_string()),
                    layer_type: "text".to_string(),
                    start_ms: Some(0),
                    end_ms: Some(3000),
                    x: Some(0.5),
                    y: Some(0.5),
                    width: Some(0.8),
                    height: Some(0.2),
                    properties: Some(serde_json::json!({
                        "content": title,
                        "font_size": 72,
                        "color": "#FFFFFF",
                        "text_align": "center",
                    })),
                }).await?;
            },
            _ => {}
        }

        info!("Applied template {} to project {}", template_id, project_id);
        Ok(())
    }

    pub async fn add_transition(
        &self,
        from_clip_id: Uuid,
        to_clip_id: Uuid,
        transition_type: &str,
        duration_ms: i64,
    ) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_conn()?;

        diesel::update(video_clips::table.find(from_clip_id))
            .set(video_clips::transition_out.eq(Some(format!("{}:{}", transition_type, duration_ms))))
            .execute(&mut conn)?;

        diesel::update(video_clips::table.find(to_clip_id))
            .set(video_clips::transition_in.eq(Some(format!("{}:{}", transition_type, duration_ms))))
            .execute(&mut conn)?;

        info!("Added {} transition between clips {} and {}", transition_type, from_clip_id, to_clip_id);
        Ok(())
    }

    pub async fn start_export(
        &self,
        project_id: Uuid,
        req: ExportRequest,
        cache: Option<&Arc<redis::Client>>,
    ) -> Result<VideoExport, diesel::result::Error> {
        let mut conn = self.get_conn()?;

        let export = VideoExport {
            id: Uuid::new_v4(),
            project_id,
            format: req.format.unwrap_or_else(|| "mp4".to_string()),
            quality: req.quality.unwrap_or_else(|| "high".to_string()),
            status: ExportStatus::Pending.to_string(),
            progress: 0,
            output_url: None,
            gbdrive_path: None,
            error_message: None,
            created_at: Utc::now(),
            completed_at: None,
        };

        diesel::insert_into(video_exports::table)
            .values(&export)
            .execute(&mut conn)?;

        diesel::update(video_projects::table.find(project_id))
            .set(video_projects::status.eq(ProjectStatus::Exporting.to_string()))
            .execute(&mut conn)?;

        if let Some(redis_client) = cache {
            if let Ok(mut redis_conn) = redis_client.get_connection() {
                let job = serde_json::json!({
                    "export_id": export.id.to_string(),
                    "project_id": project_id.to_string(),
                    "format": &export.format,
                    "quality": &export.quality,
                    "save_to_library": req.save_to_library.unwrap_or(true),
                    "created_at": Utc::now().to_rfc3339(),
                });
                let _: Result<i64, _> = redis::cmd("LPUSH")
                    .arg("video:export:queue")
                    .arg(job.to_string())
                    .query(&mut redis_conn);
                info!("Queued export job {} to Valkey", export.id);
            }
        }

        info!("Started export for project {project_id}: {}", export.id);
        Ok(export)
    }

    pub async fn get_export_status(&self, export_id: Uuid) -> Result<VideoExport, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        video_exports::table.find(export_id).first(&mut conn)
    }

    async fn recalculate_duration(&self, project_id: Uuid) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_conn()?;

        let clips: Vec<VideoClip> = video_clips::table
            .filter(video_clips::project_id.eq(project_id))
            .load(&mut conn)?;

        let max_duration = clips
            .iter()
            .map(|c| c.start_ms + c.duration_ms)
            .max()
            .unwrap_or(0);

        diesel::update(video_projects::table.find(project_id))
            .set((
                video_projects::total_duration_ms.eq(max_duration),
                video_projects::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub async fn remove_background(
        &self,
        project_id: Uuid,
        clip_id: Uuid,
        replacement: Option<String>,
    ) -> Result<BackgroundRemovalResponse, Box<dyn std::error::Error + Send + Sync>> {
        let clips = self.get_clips(project_id).await?;
        let clip = clips.iter().find(|c| c.id == clip_id).ok_or("Clip not found")?;

        let botmodels_url =
            std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/video/remove-background", botmodels_url))
            .json(&serde_json::json!({
                "video_url": &clip.source_url,
                "replacement": replacement,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Background removal failed: {error}").into());
        }

        let result: serde_json::Value = response.json().await?;

        Ok(BackgroundRemovalResponse {
            processed_url: result["processed_url"].as_str().unwrap_or("").to_string(),
            duration_ms: result["duration_ms"].as_i64().unwrap_or(0),
        })
    }

    pub async fn enhance_video(
        &self,
        project_id: Uuid,
        req: VideoEnhanceRequest,
    ) -> Result<VideoEnhanceResponse, Box<dyn std::error::Error + Send + Sync>> {
        let clips = self.get_clips(project_id).await?;
        let clip = clips.iter().find(|c| c.id == req.clip_id).ok_or("Clip not found")?;

        let botmodels_url =
            std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/video/enhance", botmodels_url))
            .json(&serde_json::json!({
                "video_url": &clip.source_url,
                "upscale_factor": req.upscale_factor,
                "denoise": req.denoise,
                "stabilize": req.stabilize,
                "color_correct": req.color_correct,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Enhancement failed: {error}").into());
        }

        let result: serde_json::Value = response.json().await?;

        Ok(VideoEnhanceResponse {
            enhanced_url: result["enhanced_url"].as_str().unwrap_or("").to_string(),
            enhancements_applied: result["enhancements"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
        })
    }

    pub async fn detect_beats(
        &self,
        project_id: Uuid,
        audio_track_id: Uuid,
        sensitivity: Option<f32>,
    ) -> Result<BeatSyncResponse, Box<dyn std::error::Error + Send + Sync>> {
        let tracks = self.get_audio_tracks(project_id).await?;
        let track = tracks.iter().find(|t| t.id == audio_track_id).ok_or("Audio track not found")?;

        let botmodels_url =
            std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/audio/detect-beats", botmodels_url))
            .json(&serde_json::json!({
                "audio_url": &track.source_url,
                "sensitivity": sensitivity.unwrap_or(0.5),
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Beat detection failed: {error}").into());
        }

        let result: serde_json::Value = response.json().await?;

        let beats: Vec<BeatMarker> = result["beats"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|b| BeatMarker {
                time_ms: b["time_ms"].as_i64().unwrap_or(0),
                strength: b["strength"].as_f64().unwrap_or(1.0) as f32,
                beat_type: b["type"].as_str().unwrap_or("beat").to_string(),
            })
            .collect();

        Ok(BeatSyncResponse {
            beats,
            tempo_bpm: result["tempo_bpm"].as_f64().unwrap_or(120.0) as f32,
        })
    }

    pub async fn generate_waveform(
        &self,
        project_id: Uuid,
        audio_track_id: Uuid,
        samples_per_second: Option<i32>,
    ) -> Result<WaveformResponse, Box<dyn std::error::Error + Send + Sync>> {
        let tracks = self.get_audio_tracks(project_id).await?;
        let track = tracks.iter().find(|t| t.id == audio_track_id).ok_or("Audio track not found")?;

        let botmodels_url =
            std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/audio/waveform", botmodels_url))
            .json(&serde_json::json!({
                "audio_url": &track.source_url,
                "samples_per_second": samples_per_second.unwrap_or(10),
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Waveform generation failed: {error}").into());
        }

        let result: serde_json::Value = response.json().await?;

        Ok(WaveformResponse {
            samples: result["samples"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
                .unwrap_or_default(),
            duration_ms: result["duration_ms"].as_i64().unwrap_or(0),
            sample_rate: result["sample_rate"].as_i64().unwrap_or(10) as i32,
        })
    }

    pub async fn process_chat_command(
        &self,
        project_id: Uuid,
        message: &str,
        playhead_ms: Option<i64>,
        selection: Option<serde_json::Value>,
    ) -> Result<ChatEditResponse, Box<dyn std::error::Error + Send + Sync>> {
        let lower_msg = message.to_lowercase();
        let mut commands_executed = Vec::new();

        if lower_msg.contains("add text") || lower_msg.contains("add title") {
            let content = extract_quoted_text(message).unwrap_or_else(|| "Text".to_string());
            let at_ms = playhead_ms.unwrap_or(0);

            self.add_layer(
                project_id,
                AddLayerRequest {
                    name: Some("Text".to_string()),
                    layer_type: "text".to_string(),
                    start_ms: Some(at_ms),
                    end_ms: Some(at_ms + 5000),
                    x: Some(0.5),
                    y: Some(0.5),
                    width: Some(0.8),
                    height: Some(0.2),
                    properties: Some(serde_json::json!({
                        "content": content,
                        "font_family": "Arial",
                        "font_size": 48,
                        "color": "#FFFFFF",
                    })),
                },
            )
            .await?;

            commands_executed.push("Added text layer".to_string());
        }

        if lower_msg.contains("delete") || lower_msg.contains("remove") {
            if let Some(sel) = &selection {
                if let Some(layer_id) = sel.get("layer_id").and_then(|v| v.as_str()) {
                    if let Ok(id) = Uuid::parse_str(layer_id) {
                        self.delete_layer(id).await?;
                        commands_executed.push("Deleted layer".to_string());
                    }
                } else if let Some(clip_id) = sel.get("clip_id").and_then(|v| v.as_str()) {
                    if let Ok(id) = Uuid::parse_str(clip_id) {
                        self.delete_clip(id).await?;
                        commands_executed.push("Deleted clip".to_string());
                    }
                }
            }
        }

        if lower_msg.contains("split") {
            if let Some(sel) = &selection {
                if let Some(clip_id) = sel.get("clip_id").and_then(|v| v.as_str()) {
                    if let Ok(id) = Uuid::parse_str(clip_id) {
                        let at = playhead_ms.unwrap_or(0);
                        self.split_clip(id, at).await?;
                        commands_executed.push("Split clip".to_string());
                    }
                }
            }
        }

        if lower_msg.contains("bigger") || lower_msg.contains("larger") {
            if let Some(sel) = &selection {
                if let Some(layer_id) = sel.get("layer_id").and_then(|v| v.as_str()) {
                    if let Ok(id) = Uuid::parse_str(layer_id) {
                        let layer = video_layers::table.find(id).first::<VideoLayer>(&mut self.get_conn()?)?;
                        self.update_layer(
                            id,
                            UpdateLayerRequest {
                                width: Some(layer.width * 1.2),
                                height: Some(layer.height * 1.2),
                                ..Default::default()
                            },
                        )
                        .await?;
                        commands_executed.push("Made layer bigger".to_string());
                    }
                }
            }
        }

        if lower_msg.contains("smaller") {
            if let Some(sel) = &selection {
                if let Some(layer_id) = sel.get("layer_id").and_then(|v| v.as_str()) {
                    if let Ok(id) = Uuid::parse_str(layer_id) {
                        let layer = video_layers::table.find(id).first::<VideoLayer>(&mut self.get_conn()?)?;
                        self.update_layer(
                            id,
                            UpdateLayerRequest {
                                width: Some(layer.width * 0.8),
                                height: Some(layer.height * 0.8),
                                ..Default::default()
                            },
                        )
                        .await?;
                        commands_executed.push("Made layer smaller".to_string());
                    }
                }
            }
        }

        let response_message = if commands_executed.is_empty() {
            "I couldn't understand that command. Try: add text \"Hello\", delete, split, make it bigger/smaller".to_string()
        } else {
            commands_executed.join(", ")
        };

        let project_detail = self.get_project_detail(project_id).await.ok();

        Ok(ChatEditResponse {
            success: !commands_executed.is_empty(),
            message: response_message,
            commands_executed,
            project: project_detail,
        })
    }
}

fn extract_quoted_text(message: &str) -> Option<String> {
    let chars: Vec<char> = message.chars().collect();
    let mut start = None;
    let mut end = None;

    for (i, c) in chars.iter().enumerate() {
        if *c == '"' || *c == '\'' || *c == '"' || *c == '"' {
            if start.is_none() {
                start = Some(i + 1);
            } else {
                end = Some(i);
                break;
            }
        }
    }

    match (start, end) {
        (Some(s), Some(e)) if e > s => Some(chars[s..e].iter().collect()),
        _ => None,
    }
}
