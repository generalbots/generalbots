// Canvas and element CRUD handlers (#1370 split), now split again by subject
// (#1730): row/domain conversion (`convert`), canvas routes (`canvases`) and
// element routes (`elements`). The `pub(crate)` handlers stay reachable at
// `api::*` through the re-exports below, which the crate root glob picks up.

mod canvases;
mod convert;
mod elements;

pub(crate) use canvases::{create_canvas, delete_canvas, get_canvas, list_canvases, update_canvas};
pub(crate) use convert::{db_to_canvas, db_to_canvas_element};
pub(crate) use elements::{create_element, delete_element, list_elements, update_element};
