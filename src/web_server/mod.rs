use actix_files::Files;
use actix_web::{HttpRequest, HttpResponse, Result};
use log::{debug, error};
use std::{fs, path::Path};

#[actix_web::get("/")]
async fn index() -> Result<HttpResponse> {
    match fs::read_to_string("web/desktop/index.html") {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(e) => {
            error!("Failed to load index page: {}", e);
            Ok(HttpResponse::InternalServerError().body("Failed to load index page"))
        }
    }
}


pub fn configure_app(cfg: &mut actix_web::web::ServiceConfig) {
    let static_path = Path::new("./web/desktop");

    // Serve all JS files
    cfg.service(
        Files::new("/js", static_path.join("js"))
            .prefer_utf8(true)
            .use_last_modified(true)
            .use_etag(true)
    );

    // Serve CSS files
    cfg.service(
        Files::new("/css", static_path.join("css"))
            .prefer_utf8(true)
            .use_last_modified(true)
            .use_etag(true)
    );

    cfg.service(
        Files::new("/drive", static_path.join("drive"))
            .prefer_utf8(true)
            .use_last_modified(true)
            .use_etag(true)
    );

    cfg.service(
        Files::new("/chat", static_path.join("chat"))
            .prefer_utf8(true)
            .use_last_modified(true)
            .use_etag(true)
    );

    cfg.service(
        Files::new("/mail", static_path.join("mail"))
            .prefer_utf8(true)
            .use_last_modified(true)
            .use_etag(true)
    );

    cfg.service(
        Files::new("/tasks", static_path.join("tasks"))
            .prefer_utf8(true)
            .use_last_modified(true)
            .use_etag(true)
    );

    // Fallback: serve index.html for any other path to enable SPA routing
    cfg.service(
        Files::new("/", static_path)
            .index_file("index.html")
            .prefer_utf8(true)
            .use_last_modified(true)
            .use_etag(true)
    );

    cfg.service(index);
}
