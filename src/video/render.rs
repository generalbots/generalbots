use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::security::command_guard::SafeCommand;
use crate::shared::utils::DbPool;

use super::models::*;
use super::schema::*;
use super::websocket::broadcast_export_progress;

pub struct VideoRenderWorker {
    db: DbPool,
    cache: Arc<redis::Client>,
    output_dir: String,
}

impl VideoRenderWorker {
    pub fn new(db: DbPool, cache: Arc<redis::Client>, output_dir: String) -> Self {
        Self {
            db,
            cache,
            output_dir,
        }
    }

    pub async fn start(self) {
        info!("Starting video render worker");
        tokio::spawn(async move {
            self.run_worker_loop().await;
        });
    }

    pub async fn run_worker_loop(&self) {
        loop {
            match self.process_next_job().await {
                Ok(true) => continue,
                Ok(false) => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
                Err(e) => {
                    error!("Worker error: {e}");
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    async fn process_next_job(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.cache.get_connection()?;

        let job_json: Option<String> = redis::cmd("RPOP")
            .arg("video:export:queue")
            .query(&mut conn)?;

        let job_str = match job_json {
            Some(j) => j,
            None => return Ok(false),
        };

        let job: serde_json::Value = serde_json::from_str(&job_str)?;
        let export_id = Uuid::parse_str(job["export_id"].as_str().unwrap_or_default())?;
        let project_id = Uuid::parse_str(job["project_id"].as_str().unwrap_or_default())?;
        let format = job["format"].as_str().unwrap_or("mp4");
        let quality = job["quality"].as_str().unwrap_or("high");
        let save_to_library = job["save_to_library"].as_bool().unwrap_or(true);
        let bot_name = job["bot_name"].as_str().map(|s| s.to_string());

        info!("Processing export job: {export_id}");

        self.update_progress(export_id, project_id, 10, "processing", None, None)
            .await?;

        match self
            .render_video(project_id, export_id, format, quality)
            .await
        {
            Ok(output_url) => {
                let gbdrive_path = if save_to_library {
                    self.save_to_gbdrive(&output_url, project_id, export_id, format, bot_name.as_deref())
                        .await
                        .ok()
                } else {
                    None
                };

                self.update_progress(
                    export_id,
                    project_id,
                    100,
                    "completed",
                    Some(output_url),
                    gbdrive_path,
                )
                .await?;
                info!("Export {export_id} completed");
            }
            Err(e) => {
                let error_msg = format!("Render failed: {e}");
                self.update_progress(export_id, project_id, 0, "failed", None, None)
                    .await?;
                error!("Export {export_id} failed: {error_msg}");
                self.set_export_error(export_id, &error_msg).await?;
            }
        }

        Ok(true)
    }

    async fn update_progress(
        &self,
        export_id: Uuid,
        project_id: Uuid,
        progress: i32,
        status: &str,
        output_url: Option<String>,
        gbdrive_path: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut db_conn = self.db.get()?;

        let completed_at = if status == "completed" || status == "failed" {
            Some(Utc::now())
        } else {
            None
        };

        diesel::update(video_exports::table.find(export_id))
            .set((
                video_exports::progress.eq(progress),
                video_exports::status.eq(status),
                video_exports::output_url.eq(&output_url),
                video_exports::gbdrive_path.eq(&gbdrive_path),
                video_exports::completed_at.eq(completed_at),
            ))
            .execute(&mut db_conn)?;

        if status == "completed" || status == "failed" {
            let new_status = if status == "completed" {
                "published"
            } else {
                "draft"
            };
            diesel::update(video_projects::table.find(project_id))
                .set(video_projects::status.eq(new_status))
                .execute(&mut db_conn)?;
        }

        broadcast_export_progress(
            export_id,
            project_id,
            status,
            progress,
            Some(format!("Export {progress}%")),
            output_url,
            gbdrive_path,
        );

        Ok(())
    }

    async fn set_export_error(
        &self,
        export_id: Uuid,
        error_message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut db_conn = self.db.get()?;

        diesel::update(video_exports::table.find(export_id))
            .set(video_exports::error_message.eq(Some(error_message)))
            .execute(&mut db_conn)?;

        Ok(())
    }

    async fn render_video(
        &self,
        project_id: Uuid,
        export_id: Uuid,
        format: &str,
        quality: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut db_conn = self.db.get()?;

        let project: VideoProject = video_projects::table.find(project_id).first(&mut db_conn)?;

        let clips: Vec<VideoClip> = video_clips::table
            .filter(video_clips::project_id.eq(project_id))
            .order(video_clips::clip_order.asc())
            .load(&mut db_conn)?;

        let layers: Vec<VideoLayer> = video_layers::table
            .filter(video_layers::project_id.eq(project_id))
            .order(video_layers::track_index.asc())
            .load(&mut db_conn)?;

        if clips.is_empty() {
            return Err("No clips in project".into());
        }

        std::fs::create_dir_all(&self.output_dir)?;

        let output_filename = format!("{export_id}.{format}");
        let output_path = format!("{}/{output_filename}", self.output_dir);

        let resolution = match quality {
            "4k" => "3840x2160",
            "high" => "1920x1080",
            "medium" => "1280x720",
            "low" => "854x480",
            _ => "1920x1080",
        };

        let bitrate = match quality {
            "4k" => "20M",
            "high" => "8M",
            "medium" => "4M",
            "low" => "2M",
            _ => "8M",
        };

        let filter_complex = self.build_filter_complex(&clips, &layers, &project, resolution);

        let mut cmd = SafeCommand::new("ffmpeg")
            .map_err(|e| format!("Failed to create command: {e}"))?;

        cmd.arg("-y").map_err(|e| format!("Arg error: {e}"))?;

        for clip in &clips {
            cmd.arg("-i").map_err(|e| format!("Arg error: {e}"))?;
            cmd.arg(&clip.source_url).map_err(|e| format!("Arg error: {e}"))?;
        }

        if !filter_complex.is_empty() {
            cmd.arg("-filter_complex").map_err(|e| format!("Arg error: {e}"))?;
            cmd.arg(&filter_complex).map_err(|e| format!("Arg error: {e}"))?;
            cmd.arg("-map").map_err(|e| format!("Arg error: {e}"))?;
            cmd.arg("[outv]").map_err(|e| format!("Arg error: {e}"))?;

            if clips.len() == 1 {
                cmd.arg("-map").map_err(|e| format!("Arg error: {e}"))?;
                cmd.arg("0:a?").map_err(|e| format!("Arg error: {e}"))?;
            }
        }

        cmd.arg("-c:v").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("libx264").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("-preset").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("medium").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("-b:v").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg(bitrate).map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("-c:a").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("aac").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("-b:a").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("192k").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("-movflags").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg("+faststart").map_err(|e| format!("Arg error: {e}"))?;
        cmd.arg(&output_path).map_err(|e| format!("Arg error: {e}"))?;

        info!("Running FFmpeg render for export {export_id}");

        let result = cmd.execute().map_err(|e| format!("Execution failed: {e}"))?;

        if !result.success {
            warn!("FFmpeg stderr: {}", result.stderr);
            return Err(format!("FFmpeg failed: {}", result.stderr).into());
        }

        let output_url = format!("/video/exports/{output_filename}");
        Ok(output_url)
    }

    fn build_filter_complex(
        &self,
        clips: &[VideoClip],
        layers: &[VideoLayer],
        project: &VideoProject,
        resolution: &str,
    ) -> String {
        let mut filters = Vec::new();
        let mut inputs = Vec::new();

        for (i, clip) in clips.iter().enumerate() {
            let trim_start = clip.trim_in_ms as f64 / 1000.0;
            let trim_end = (clip.duration_ms - clip.trim_out_ms) as f64 / 1000.0;

            filters.push(format!(
                "[{i}:v]trim=start={trim_start}:end={trim_end},setpts=PTS-STARTPTS,scale={resolution}:force_original_aspect_ratio=decrease,pad={resolution}:(ow-iw)/2:(oh-ih)/2[v{i}]"
            ));
            inputs.push(format!("[v{i}]"));
        }

        if clips.len() > 1 {
            let concat_inputs = inputs.join("");
            filters.push(format!(
                "{concat_inputs}concat=n={}:v=1:a=0[outv]",
                clips.len()
            ));
        } else if !inputs.is_empty() {
            filters.push(format!("{}copy[outv]", inputs[0]));
        }

        for layer in layers {
            if layer.layer_type == "text" {
                if let Some(content) = layer
                    .properties_json
                    .get("content")
                    .and_then(|c| c.as_str())
                {
                    let font_size = layer
                        .properties_json
                        .get("font_size")
                        .and_then(|s| s.as_i64())
                        .unwrap_or(48);
                    let color = layer
                        .properties_json
                        .get("color")
                        .and_then(|c| c.as_str())
                        .unwrap_or("white");

                    let x = (layer.x * project.resolution_width as f32) as i32;
                    let y = (layer.y * project.resolution_height as f32) as i32;

                    let escaped_content = content
                        .replace('\'', "'\\''")
                        .replace(':', "\\:")
                        .replace('\\', "\\\\");

                    filters.push(format!(
                        "[outv]drawtext=text='{}':fontsize={}:fontcolor={}:x={}:y={}:enable='between(t,{},{})':alpha={}[outv]",
                        escaped_content,
                        font_size,
                        color.trim_start_matches('#'),
                        x,
                        y,
                        layer.start_ms as f64 / 1000.0,
                        layer.end_ms as f64 / 1000.0,
                        layer.opacity
                    ));
                }
            }
        }

        if filters.is_empty() {
            return String::new();
        }

        filters.join(";")
    }

    async fn save_to_gbdrive(
        &self,
        output_url: &str,
        project_id: Uuid,
        export_id: Uuid,
        format: &str,
        bot_name: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut db_conn = self.db.get()?;

        let project: VideoProject = video_projects::table.find(project_id).first(&mut db_conn)?;

        let safe_name: String = project
            .name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{safe_name}_{timestamp}.{format}");
        let gbdrive_path = format!("videos/{filename}");

        let source_path = format!(
            "{}/{}",
            self.output_dir,
            output_url.trim_start_matches("/video/exports/")
        );

        if std::env::var("S3_ENDPOINT").is_ok() {
            let bot = bot_name.unwrap_or("default");
            let bucket = format!("{bot}.gbai");
            let key = format!("{bot}.gbdrive/{gbdrive_path}");

            info!("Uploading video to S3: s3://{bucket}/{key}");

            let file_data = std::fs::read(&source_path)?;

            let s3_config = aws_config::from_env().load().await;
            let s3_client = aws_sdk_s3::Client::new(&s3_config);

            s3_client
                .put_object()
                .bucket(&bucket)
                .key(&key)
                .content_type(format!("video/{format}"))
                .body(file_data.into())
                .send()
                .await
                .map_err(|e| format!("S3 upload failed: {e}"))?;

            info!("Video saved to .gbdrive: {gbdrive_path}");
        } else {
            let gbdrive_dir = std::env::var("GBDRIVE_DIR").unwrap_or_else(|_| "./.gbdrive".to_string());
            let videos_dir = format!("{gbdrive_dir}/videos");

            std::fs::create_dir_all(&videos_dir)?;

            let dest_path = format!("{videos_dir}/{filename}");
            std::fs::copy(&source_path, &dest_path)?;

            info!("Video saved to local .gbdrive: {gbdrive_path}");
        }

        diesel::update(video_exports::table.find(export_id))
            .set(video_exports::gbdrive_path.eq(Some(&gbdrive_path)))
            .execute(&mut db_conn)?;

        Ok(gbdrive_path)
    }
}

pub fn start_render_worker(db: DbPool, cache: Arc<redis::Client>, output_dir: String) {
    let worker = VideoRenderWorker::new(db, cache, output_dir);
    tokio::spawn(async move {
        worker.run_worker_loop().await;
    });
}
