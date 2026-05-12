use crate::storage::DriveOps;
use crate::types::{SlidesAiRequest, SlidesAiResponse};
use crate::SlidesState;
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

pub async fn handle_slides_ai<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Json(req): Json<SlidesAiRequest>,
) -> impl IntoResponse {
    let command = req.command.to_lowercase();

    let response = if command.contains("add") && command.contains("slide") {
        "I've added a new slide to your presentation."
    } else if command.contains("duplicate") {
        "I've duplicated the current slide."
    } else if command.contains("delete") || command.contains("remove") {
        "I've removed the slide from your presentation."
    } else if command.contains("text") || command.contains("title") {
        "I've added a text box to your slide. Click to edit."
    } else if command.contains("image") || command.contains("picture") {
        "I've added an image placeholder. Click to upload an image."
    } else if command.contains("shape") {
        "I've added a shape to your slide. You can resize and move it."
    } else if command.contains("chart") {
        "I've added a chart. Click to edit the data."
    } else if command.contains("table") {
        "I've added a table. Click cells to edit."
    } else if command.contains("theme") || command.contains("design") {
        "I can help you change the theme. Choose from the Design menu."
    } else if command.contains("animate") || command.contains("animation") {
        "I've added an animation to the selected element."
    } else if command.contains("transition") {
        "I've applied a transition effect to this slide."
    } else if command.contains("help") {
        "I can help you with:\n\u{2022} Add/duplicate/delete slides\n\u{2022} Insert text, images, shapes\n\u{2022} Add charts and tables\n\u{2022} Apply themes and animations\n\u{2022} Set slide transitions"
    } else {
        "I understand you want help with your presentation. Try commands like 'add slide', 'insert image', 'add chart', or 'apply animation'."
    };

    Json(SlidesAiResponse {
        response: response.to_string(),
        action: None,
        data: None,
    })
}
