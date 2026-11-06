use crate::storage::DriveOps;
use crate::types::{SlidesAiRequest, SlidesAiResponse};
use crate::SlidesState;
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// Maps a lowercased command to an honest reply.
///
/// This endpoint does not mutate the deck, so no branch may claim that an
/// action was performed. Each branch explains the manual control instead.
fn ai_reply_for(command: &str) -> &'static str {
    if command.contains("add") && command.contains("slide") {
        "This chat endpoint cannot change the deck yet. Use the + button above the slide list or the slide insert menu to add a slide."
    } else if command.contains("duplicate") {
        "This chat endpoint cannot change the deck yet. Select the slide in the slide list and use the duplicate button or the slide menu."
    } else if command.contains("delete") || command.contains("remove") {
        "This chat endpoint cannot change the deck yet. Select the slide in the slide list and use delete or the slide menu to remove it."
    } else if command.contains("text") || command.contains("title") {
        "This chat endpoint cannot insert elements yet. Use Insert \u{2192} Text box (or the text tool in the toolbar), then click the slide to edit its title or body."
    } else if command.contains("image") || command.contains("picture") {
        "This chat endpoint cannot insert images yet. Use Insert \u{2192} Image, then choose a file to upload and place it on the slide."
    } else if command.contains("shape") {
        "This chat endpoint cannot insert shapes yet. Use Insert \u{2192} Shape, then drag on the slide to place and resize it."
    } else if command.contains("chart") {
        "This chat endpoint cannot insert charts yet. Use Insert \u{2192} Chart, then edit the chart data in the panel that opens."
    } else if command.contains("table") {
        "This chat endpoint cannot insert tables yet. Use Insert \u{2192} Table, then click cells to edit them."
    } else if command.contains("theme") || command.contains("design") {
        "This chat endpoint cannot change the theme yet. Open the Design menu to choose a theme for the presentation."
    } else if command.contains("animate") || command.contains("animation") {
        "This chat endpoint cannot add animations yet. Select the element and use the Animation panel to choose an effect."
    } else if command.contains("transition") {
        "This chat endpoint cannot set transitions yet. Select the slide and use the Transition panel to choose an effect."
    } else if command.contains("help") {
        "This assistant only explains how to use the editor; it cannot modify the deck. The editor can help you:\n\u{2022} Add, duplicate and delete slides from the slide list\n\u{2022} Insert text, images, shapes, charts and tables from the Insert menu\n\u{2022} Apply themes from the Design menu\n\u{2022} Add animations and transitions from their panels"
    } else {
        "This assistant only explains how to use the editor; it cannot modify the deck. Try 'add slide', 'insert image', 'add chart' or 'apply animation' to get directions for those controls."
    }
}

pub async fn handle_slides_ai<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Json(req): Json<SlidesAiRequest>,
) -> impl IntoResponse {
    let command = req.command.to_lowercase();
    let response = ai_reply_for(&command);

    Json(SlidesAiResponse {
        response: response.to_string(),
        action: None,
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTION_COMMANDS: &[&str] = &[
        "add slide",
        "duplicate slide",
        "delete slide",
        "remove slide",
        "add text",
        "insert image",
        "add shape",
        "add chart",
        "add table",
        "apply theme",
        "apply animation",
        "set transition",
    ];

    fn assert_no_false_confirmation(command: &str, reply: &str) {
        assert!(
            !reply.contains(concat!("I", "'ve")) && !reply.contains(concat!("I", " have")),
            "false confirmation for {command}: {reply}"
        );
    }

    #[test]
    fn mutation_commands_explain_controls_without_claiming_success() {
        for command in ACTION_COMMANDS {
            let reply = ai_reply_for(command);
            assert_no_false_confirmation(command, reply);
            assert!(
                reply.contains("This chat endpoint cannot"),
                "missing honest limitation for {command}: {reply}"
            );
        }
    }

    #[test]
    fn action_commands_point_to_manual_controls() {
        assert!(ai_reply_for("add slide").contains("+ button"));
        assert!(ai_reply_for("duplicate slide").contains("duplicate button"));
        assert!(ai_reply_for("insert image").contains("Insert"));
        assert!(ai_reply_for("apply theme").contains("Design menu"));
        assert!(ai_reply_for("help").contains("cannot modify the deck"));
    }

    #[test]
    fn unknown_command_is_instruction_only() {
        let reply = ai_reply_for("make it prettier");
        assert!(!reply.contains(concat!("I", "'ve")) && !reply.contains(concat!("I", " have")));
        assert!(reply.contains("cannot modify the deck"));
    }
}
