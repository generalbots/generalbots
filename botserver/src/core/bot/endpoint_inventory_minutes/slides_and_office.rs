//! Slides, social, system, task, tax and template routes of the API catalogue.

use crate::core::bot::api_catalog::ApiEndpoint;

#[rustfmt::skip]
pub(crate) const ROUTES: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/:id",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/ai",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/cursor",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/cursors",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/delete",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/element/add",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/element/delete",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/element/update",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/export",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/import",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/list",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/load",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/media",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/media/delete",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/media/list",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/media/update",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/mentions/{user_id}",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/new",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/presenter/end",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/presenter/notes",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/presenter/start",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/presenter/update",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/save",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/search",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/selection",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/selections",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/slide/add",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/slide/delete",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/slide/duplicate",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/slide/notes",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/slide/reorder",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/theme",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/transition",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/transition/all",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/slides/transition/remove",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/{id}/collaborators",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/{presentation_id}/presence",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/{presentation_id}/selections",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/slides/{presentation_id}/typing",
        summary: "registered in crates/botslides/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/social/analytics",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/social/communities",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/social/communities/:id",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/social/communities/:id/join",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/social/communities/:id/leave",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/social/feed",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/social/posts",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/social/posts/:id",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/social/posts/:id/comments",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/social/posts/:id/react",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/social/posts/:id/react/:type",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/social/praise",
        summary: "registered in crates/botsocial/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/system/check-updates",
        summary: "registered in crates/botapi/src/system.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/system/versions",
        summary: "registered in crates/botapi/src/system.rs",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/tasks/:task_id",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/tasks/:task_id/dependencies",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/tasks/:task_id/progress",
        summary: "registered in crates/botproject/src/lib.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/tax/calculate",
        summary: "registered in crates/bottax/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tax/cte",
        summary: "registered in crates/bottax/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tax/nfe",
        summary: "registered in crates/bottax/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/tax/nfe/:id/authorize",
        summary: "registered in crates/bottax/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tax/nfse",
        summary: "registered in crates/bottax/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/tax/sped",
        summary: "registered in crates/bottax/src/routes.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/telegram/send",
        summary: "registered in crates/bottelegram/src/webhook.rs",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/templates/deploy/:id",
        summary: "registered in crates/bottemplates/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/templates/list",
        summary: "registered in crates/bottemplates/src/routes.rs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/templates/preview/:id",
        summary: "registered in crates/bottemplates/src/routes.rs",
    },
];
