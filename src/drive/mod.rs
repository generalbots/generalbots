use crate::shared::state::AppState;
use crate::ui_tree::file_tree::{FileTree, TreeNode};
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct FileItem {
    name: String,
    path: String,
    is_dir: bool,
    icon: String,
}

#[derive(Deserialize)]
pub struct ListQuery {
    path: Option<String>,
    bucket: Option<String>,
}

#[derive(Deserialize)]
pub struct ReadRequest {
    bucket: String,
    path: String,
}

#[derive(Deserialize)]
pub struct WriteRequest {
    bucket: String,
    path: String,
    content: String,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    bucket: String,
    path: String,
}

#[derive(Deserialize)]
pub struct CreateFolderRequest {
    bucket: String,
    path: String,
    name: String,
}

async fn list_files(
    query: web::Query<ListQuery>,
    app_state: web::Data<Arc<AppState>>,
) -> impl Responder {
    let mut tree = FileTree::new(app_state.get_ref().clone());

    let result = if let Some(bucket) = &query.bucket {
        if let Some(path) = &query.path {
            tree.enter_folder(bucket.clone(), path.clone()).await
        } else {
            tree.enter_bucket(bucket.clone()).await
        }
    } else {
        tree.load_root().await
    };

    if let Err(e) = result {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }));
    }

    let items: Vec<FileItem> = tree
        .render_items()
        .iter()
        .map(|(display, node)| {
            let (name, path, is_dir, icon) = match node {
                TreeNode::Bucket { name } => {
                    let icon = if name.ends_with(".gbai") {
                        "🤖"
                    } else {
                        "📦"
                    };
                    (name.clone(), name.clone(), true, icon.to_string())
                }
                TreeNode::Folder { bucket, path } => {
                    let name = path.split('/').last().unwrap_or(path).to_string();
                    (name, path.clone(), true, "📁".to_string())
                }
                TreeNode::File { bucket, path } => {
                    let name = path.split('/').last().unwrap_or(path).to_string();
                    let icon = if path.ends_with(".bas") {
                        "⚙️"
                    } else if path.ends_with(".ast") {
                        "🔧"
                    } else if path.ends_with(".csv") {
                        "📊"
                    } else if path.ends_with(".gbkb") {
                        "📚"
                    } else if path.ends_with(".json") {
                        "🔖"
                    } else if path.ends_with(".txt") || path.ends_with(".md") {
                        "📃"
                    } else {
                        "📄"
                    };
                    (name, path.clone(), false, icon.to_string())
                }
            };

            FileItem {
                name,
                path,
                is_dir,
                icon,
            }
        })
        .collect();

    HttpResponse::Ok().json(items)
}

async fn read_file(
    req: web::Json<ReadRequest>,
    app_state: web::Data<Arc<AppState>>,
) -> impl Responder {
    if let Some(drive) = &app_state.drive {
        match drive
            .get_object()
            .bucket(&req.bucket)
            .key(&req.path)
            .send()
            .await
        {
            Ok(response) => match response.body.collect().await {
                Ok(data) => {
                    let bytes = data.into_bytes();
                    match String::from_utf8(bytes.to_vec()) {
                        Ok(content) => HttpResponse::Ok().json(serde_json::json!({
                            "content": content
                        })),
                        Err(_) => HttpResponse::BadRequest().json(serde_json::json!({
                            "error": "File is not valid UTF-8 text"
                        })),
                    }
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                })),
            },
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Drive not connected"
        }))
    }
}

async fn write_file(
    req: web::Json<WriteRequest>,
    app_state: web::Data<Arc<AppState>>,
) -> impl Responder {
    if let Some(drive) = &app_state.drive {
        match drive
            .put_object()
            .bucket(&req.bucket)
            .key(&req.path)
            .body(req.content.clone().into_bytes().into())
            .send()
            .await
        {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                "success": true
            })),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Drive not connected"
        }))
    }
}

async fn delete_file(
    req: web::Json<DeleteRequest>,
    app_state: web::Data<Arc<AppState>>,
) -> impl Responder {
    if let Some(drive) = &app_state.drive {
        match drive
            .delete_object()
            .bucket(&req.bucket)
            .key(&req.path)
            .send()
            .await
        {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                "success": true
            })),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Drive not connected"
        }))
    }
}

async fn create_folder(
    req: web::Json<CreateFolderRequest>,
    app_state: web::Data<Arc<AppState>>,
) -> impl Responder {
    if let Some(drive) = &app_state.drive {
        let folder_path = if req.path.is_empty() {
            format!("{}/", req.name)
        } else {
            format!("{}/{}/", req.path, req.name)
        };

        match drive
            .put_object()
            .bucket(&req.bucket)
            .key(&folder_path)
            .body(Vec::new().into())
            .send()
            .await
        {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                "success": true
            })),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Drive not connected"
        }))
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/files")
            .route("/list", web::get().to(list_files))
            .route("/read", web::post().to(read_file))
            .route("/write", web::post().to(write_file))
            .route("/delete", web::post().to(delete_file))
            .route("/create-folder", web::post().to(create_folder)),
    );
}
