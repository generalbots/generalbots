use crate::engine::VideoEngine;
use crate::models::*;
use crate::requests::*;
use crate::responses::*;
use crate::schema::*;

use chrono::Utc;
use diesel::prelude::*;
use tracing::info;
use uuid::Uuid;

impl VideoEngine {
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
}
