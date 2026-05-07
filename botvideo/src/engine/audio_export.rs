use crate::engine::VideoEngine;
use crate::models::*;
use crate::requests::*;
use crate::schema::*;

use chrono::Utc;
use diesel::prelude::*;
use tracing::info;
use uuid::Uuid;

impl VideoEngine {
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
        cache: Option<&std::sync::Arc<redis::Client>>,
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

    pub async fn recalculate_duration(&self, project_id: Uuid) -> Result<(), diesel::result::Error> {
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
}

use crate::responses::{ExportStatus, ProjectStatus};
