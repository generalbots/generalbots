use super::super::types::{ActionTemplate, Parameter, ParameterType};
use super::{
    destructive, param, read, write, CREATE_PARAMS, DELETE_PARAMS, GET_PARAMS, LIST_PARAMS,
    SEARCH_PARAMS, UPDATE_PARAMS,
};

const DATE_RANGE: &[Parameter] = &[
    param("start", ParameterType::DateTime, false, "Range start"),
    param("end", ParameterType::DateTime, false, "Range end"),
];
const FILE_LIST: &[Parameter] = &[
    param(
        "folder_id",
        ParameterType::String,
        false,
        "Folder identifier or path",
    ),
    param(
        "limit",
        ParameterType::Integer,
        false,
        "Maximum number of results",
    ),
];
const FILE_GET: &[Parameter] = &[param(
    "file_id",
    ParameterType::String,
    true,
    "Provider file identifier or path",
)];
const FILE_UPLOAD: &[Parameter] = &[
    param(
        "folder_id",
        ParameterType::String,
        false,
        "Destination folder",
    ),
    param("name", ParameterType::String, true, "File name"),
    param(
        "content_reference",
        ParameterType::String,
        true,
        "Reference to content already available to the backend",
    ),
];
const SHEET_RANGE: &[Parameter] = &[
    param(
        "spreadsheet_id",
        ParameterType::String,
        true,
        "Spreadsheet identifier",
    ),
    param("range", ParameterType::String, true, "A1 range"),
];
const SHEET_WRITE: &[Parameter] = &[
    param(
        "spreadsheet_id",
        ParameterType::String,
        true,
        "Spreadsheet identifier",
    ),
    param("range", ParameterType::String, true, "A1 range"),
    param(
        "values",
        ParameterType::Json,
        true,
        "Two-dimensional values array",
    ),
];
const TASK_COMPLETE: &[Parameter] = &[param(
    "task_id",
    ParameterType::String,
    true,
    "Task identifier",
)];

pub(crate) const CALENDAR_ACTIONS: &[ActionTemplate] = &[
    read(
        "events.list",
        "list",
        "List events",
        "List events in a time range.",
        DATE_RANGE,
    ),
    read(
        "events.search",
        "search",
        "Search events",
        "Search calendar events.",
        SEARCH_PARAMS,
    ),
    write(
        "events.create",
        "create",
        "Create event",
        "Create a calendar event.",
        CREATE_PARAMS,
    ),
    write(
        "events.update",
        "update",
        "Update event",
        "Update an existing calendar event.",
        UPDATE_PARAMS,
    ),
    destructive(
        "events.delete",
        "delete",
        "Delete event",
        "Delete a calendar event.",
        DELETE_PARAMS,
    ),
];

pub(crate) const FILE_ACTIONS: &[ActionTemplate] = &[
    read(
        "files.list",
        "list",
        "List files",
        "List files in a folder.",
        FILE_LIST,
    ),
    read(
        "files.search",
        "search",
        "Search files",
        "Search files and folders.",
        SEARCH_PARAMS,
    ),
    read(
        "files.get",
        "get",
        "Get file",
        "Read file metadata or content.",
        FILE_GET,
    ),
    write(
        "files.upload",
        "upload",
        "Upload file",
        "Upload a file into a folder.",
        FILE_UPLOAD,
    ),
    destructive(
        "files.delete",
        "delete",
        "Delete file",
        "Delete a file or folder.",
        FILE_GET,
    ),
];

pub(crate) const SPREADSHEET_ACTIONS: &[ActionTemplate] = &[
    read(
        "spreadsheets.list",
        "list",
        "List spreadsheets",
        "List available spreadsheets.",
        LIST_PARAMS,
    ),
    read(
        "values.get",
        "get",
        "Read values",
        "Read values from a spreadsheet range.",
        SHEET_RANGE,
    ),
    write(
        "values.update",
        "update",
        "Update values",
        "Replace values in a spreadsheet range.",
        SHEET_WRITE,
    ),
    write(
        "rows.append",
        "append",
        "Append rows",
        "Append rows to a spreadsheet range.",
        SHEET_WRITE,
    ),
    write(
        "spreadsheets.create",
        "create",
        "Create spreadsheet",
        "Create a spreadsheet.",
        CREATE_PARAMS,
    ),
];

pub(crate) const TASK_ACTIONS: &[ActionTemplate] = &[
    read(
        "tasks.list",
        "list",
        "List tasks",
        "List tasks with optional filters.",
        LIST_PARAMS,
    ),
    read(
        "tasks.search",
        "search",
        "Search tasks",
        "Search tasks by text.",
        SEARCH_PARAMS,
    ),
    write(
        "tasks.create",
        "create",
        "Create task",
        "Create a task.",
        CREATE_PARAMS,
    ),
    write(
        "tasks.update",
        "update",
        "Update task",
        "Update task fields.",
        UPDATE_PARAMS,
    ),
    write(
        "tasks.complete",
        "complete",
        "Complete task",
        "Mark a task complete.",
        TASK_COMPLETE,
    ),
    destructive(
        "tasks.delete",
        "delete",
        "Delete task",
        "Delete a task.",
        DELETE_PARAMS,
    ),
];

pub(crate) const KNOWLEDGE_ACTIONS: &[ActionTemplate] = &[
    read(
        "pages.search",
        "search",
        "Search pages",
        "Search pages and knowledge entries.",
        SEARCH_PARAMS,
    ),
    read(
        "pages.get",
        "get",
        "Get page",
        "Read a page and its metadata.",
        GET_PARAMS,
    ),
    write(
        "pages.create",
        "create",
        "Create page",
        "Create a knowledge page.",
        CREATE_PARAMS,
    ),
    write(
        "pages.update",
        "update",
        "Update page",
        "Update a knowledge page.",
        UPDATE_PARAMS,
    ),
    destructive(
        "pages.archive",
        "archive",
        "Archive page",
        "Archive a knowledge page.",
        DELETE_PARAMS,
    ),
];

pub(crate) const PROJECT_ACTIONS: &[ActionTemplate] = &[
    read(
        "work_items.list",
        "list",
        "List work items",
        "List project work items.",
        LIST_PARAMS,
    ),
    read(
        "work_items.search",
        "search",
        "Search work items",
        "Search issues, tasks, or cards.",
        SEARCH_PARAMS,
    ),
    write(
        "work_items.create",
        "create",
        "Create work item",
        "Create an issue, task, or card.",
        CREATE_PARAMS,
    ),
    write(
        "work_items.update",
        "update",
        "Update work item",
        "Update a project work item.",
        UPDATE_PARAMS,
    ),
    destructive(
        "work_items.delete",
        "delete",
        "Delete work item",
        "Delete a project work item.",
        DELETE_PARAMS,
    ),
];
