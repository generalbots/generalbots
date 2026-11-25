use super::AppCommand;
use std::sync::OnceLock;

mod browser_video_and_photos;
mod docs_and_sheet_crud;
mod office_create_and_utilities;
mod office_review_ops;
mod sheet_slides_and_canvas;
mod system_store_and_media;
#[cfg(feature = "consent")]
mod undo_ops;

#[cfg(test)]
mod office_parity_tests;
#[cfg(all(test, feature = "consent"))]
mod undo_parity_tests;

static ALL: OnceLock<Vec<AppCommand>> = OnceLock::new();

pub fn all() -> &'static [AppCommand] {
    ALL.get_or_init(|| {
        // The chat-native undo verb shares the consent feature gate with its
        // HTTP route, so a no-consent build never advertises a dead command.
        #[cfg(feature = "consent")]
        let undo: &[AppCommand] = undo_ops::UNDO_OPS;
        #[cfg(not(feature = "consent"))]
        let undo: &[AppCommand] = &[];
        // Order follows the historical split order so the catalog keeps the
        // same deterministic ordering of commands.
        let mut v = Vec::with_capacity(
            office_create_and_utilities::OFFICE_CREATE_AND_UTILITIES.len()
                + system_store_and_media::SYSTEM_STORE_AND_MEDIA.len()
                + browser_video_and_photos::BROWSER_VIDEO_AND_PHOTOS.len()
                + docs_and_sheet_crud::DOCS_AND_SHEET_CRUD.len()
                + sheet_slides_and_canvas::SHEET_SLIDES_AND_CANVAS.len()
                + office_review_ops::OFFICE_REVIEW_OPS.len()
                + undo.len(),
        );
        v.extend_from_slice(office_create_and_utilities::OFFICE_CREATE_AND_UTILITIES);
        v.extend_from_slice(system_store_and_media::SYSTEM_STORE_AND_MEDIA);
        v.extend_from_slice(browser_video_and_photos::BROWSER_VIDEO_AND_PHOTOS);
        v.extend_from_slice(docs_and_sheet_crud::DOCS_AND_SHEET_CRUD);
        v.extend_from_slice(sheet_slides_and_canvas::SHEET_SLIDES_AND_CANVAS);
        v.extend_from_slice(office_review_ops::OFFICE_REVIEW_OPS);
        v.extend_from_slice(undo);
        v
    })
    .as_slice()
}
