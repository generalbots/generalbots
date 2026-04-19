DROP INDEX IF EXISTS idx_canvas_comments_unresolved;
DROP INDEX IF EXISTS idx_canvas_comments_parent;
DROP INDEX IF EXISTS idx_canvas_comments_element;
DROP INDEX IF EXISTS idx_canvas_comments_canvas;

DROP INDEX IF EXISTS idx_canvas_versions_number;
DROP INDEX IF EXISTS idx_canvas_versions_canvas;

DROP INDEX IF EXISTS idx_canvas_collaborators_user;
DROP INDEX IF EXISTS idx_canvas_collaborators_canvas;

DROP INDEX IF EXISTS idx_canvas_elements_z_index;
DROP INDEX IF EXISTS idx_canvas_elements_type;
DROP INDEX IF EXISTS idx_canvas_elements_canvas;

DROP INDEX IF EXISTS idx_canvases_template;
DROP INDEX IF EXISTS idx_canvases_public;
DROP INDEX IF EXISTS idx_canvases_created_by;
DROP INDEX IF EXISTS idx_canvases_org_bot;

DROP TABLE IF EXISTS canvas_comments;
DROP TABLE IF EXISTS canvas_versions;
DROP TABLE IF EXISTS canvas_collaborators;
DROP TABLE IF EXISTS canvas_elements;
DROP TABLE IF EXISTS canvases;
