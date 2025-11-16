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

#[actix_web::get("/{botname}")]
async fn bot_index(req: HttpRequest) -> Result<HttpResponse> {
    let botname = req.match_info().query("botname");
    debug!("Serving bot interface for: {}", botname);
    match fs::read_to_string("web/desktop/index.html") {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(e) => {
            error!("Failed to load index page for bot {}: {}", botname, e);
            Ok(HttpResponse::InternalServerError().body("Failed to load index page"))
        }
    }
}

pub fn configure_app(cfg: &mut actix_web::web::ServiceConfig) {
    let static_path = Path::new("./web/desktop");
    
    // Serve all static files from desktop directory
    cfg.service(
        Files::new("/", static_path)
            .index_file("index.html")
            .prefer_utf8(true)
            .use_last_modified(true)
            .use_etag(true)
            .show_files_listing()
    );
    
    // Serve all JS files
    cfg.service(
        Files::new("/js", static_path.join("js"))
            .prefer_utf8(true)
            .use_last_modified(true)
            .use_etag(true)
    );
    
    // Serve all component directories
    ["drive", "tasks", "mail"].iter().for_each(|dir| {
        cfg.service(
            Files::new(&format!("/{}", dir), static_path.join(dir))
                .prefer_utf8(true)
                .use_last_modified(true)
                .use_etag(true)
        );
    });
    
    // Serve index routes
    cfg.service(index);
    cfg.service(bot_index);
}
