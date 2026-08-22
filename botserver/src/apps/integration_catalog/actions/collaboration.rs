use super::super::types::{ActionTemplate, Parameter, ParameterType};
use super::{
    destructive, param, read, write, CREATE_PARAMS, DELETE_PARAMS, GET_PARAMS, LIST_PARAMS,
    SEARCH_PARAMS, UPDATE_PARAMS,
};

const DATE_RANGE: &[Parameter] = &[
    param("start", ParameterType::DateTime, false, "Range start"),
    param("end", ParameterType::DateTime, false, "Range end"),
];
const SEND_MESSAGE: &[Parameter] = &[
    param(
        "destination",
        ParameterType::String,
        true,
        "Channel, thread, or recipient",
    ),
    param("message", ParameterType::String, true, "Message body"),
];
const MEETING_CREATE: &[Parameter] = &[
    param("title", ParameterType::String, true, "Meeting title"),
    param("start", ParameterType::DateTime, true, "Meeting start"),
    param(
        "attendees",
        ParameterType::Json,
        false,
        "Attendee addresses",
    ),
];
const WORKFLOW_RUN: &[Parameter] = &[
    param(
        "workflow_id",
        ParameterType::String,
        true,
        "Workflow identifier",
    ),
    param("input", ParameterType::Json, false, "Workflow input"),
];

pub(crate) const MAIL_ACTIONS: &[ActionTemplate] = &[
    read(
        "messages.list",
        "list",
        "List messages",
        "List mailbox messages.",
        LIST_PARAMS,
    ),
    read(
        "messages.search",
        "search",
        "Search messages",
        "Search mailbox messages.",
        SEARCH_PARAMS,
    ),
    read(
        "messages.get",
        "get",
        "Get message",
        "Read a message and its metadata.",
        GET_PARAMS,
    ),
    write(
        "messages.send",
        "send",
        "Send message",
        "Send an email message.",
        SEND_MESSAGE,
    ),
    destructive(
        "messages.delete",
        "delete",
        "Delete message",
        "Delete a mailbox message.",
        DELETE_PARAMS,
    ),
];

pub(crate) const MESSAGING_ACTIONS: &[ActionTemplate] = &[
    read(
        "channels.list",
        "list",
        "List channels",
        "List available channels or conversations.",
        LIST_PARAMS,
    ),
    read(
        "messages.search",
        "search",
        "Search messages",
        "Search conversation messages.",
        SEARCH_PARAMS,
    ),
    read(
        "messages.get",
        "get",
        "Get message",
        "Read a message and its thread context.",
        GET_PARAMS,
    ),
    write(
        "messages.send",
        "send",
        "Send message",
        "Send a message to a channel or recipient.",
        SEND_MESSAGE,
    ),
    destructive(
        "messages.delete",
        "delete",
        "Delete message",
        "Delete a message.",
        DELETE_PARAMS,
    ),
];

pub(crate) const MEETING_ACTIONS: &[ActionTemplate] = &[
    read(
        "meetings.list",
        "list",
        "List meetings",
        "List scheduled meetings.",
        DATE_RANGE,
    ),
    read(
        "meetings.get",
        "get",
        "Get meeting",
        "Read meeting details.",
        GET_PARAMS,
    ),
    read(
        "recordings.list",
        "list",
        "List recordings",
        "List meeting recordings.",
        GET_PARAMS,
    ),
    write(
        "meetings.create",
        "create",
        "Create meeting",
        "Schedule a meeting.",
        MEETING_CREATE,
    ),
    destructive(
        "meetings.cancel",
        "cancel",
        "Cancel meeting",
        "Cancel a scheduled meeting.",
        DELETE_PARAMS,
    ),
];

pub(crate) const FORM_ACTIONS: &[ActionTemplate] = &[
    read(
        "forms.list",
        "list",
        "List forms",
        "List forms.",
        LIST_PARAMS,
    ),
    read(
        "responses.list",
        "list",
        "List responses",
        "List responses for a form.",
        GET_PARAMS,
    ),
    write(
        "forms.create",
        "create",
        "Create form",
        "Create a form.",
        CREATE_PARAMS,
    ),
    write(
        "forms.update",
        "update",
        "Update form",
        "Update a form.",
        UPDATE_PARAMS,
    ),
    destructive(
        "forms.delete",
        "delete",
        "Delete form",
        "Delete a form.",
        DELETE_PARAMS,
    ),
];

pub(crate) const SIGNATURE_ACTIONS: &[ActionTemplate] = &[
    read(
        "envelopes.list",
        "list",
        "List envelopes",
        "List signature envelopes.",
        LIST_PARAMS,
    ),
    read(
        "envelopes.get",
        "get",
        "Get envelope",
        "Read envelope status and recipients.",
        GET_PARAMS,
    ),
    write(
        "envelopes.create",
        "create",
        "Create envelope",
        "Create a signature envelope.",
        CREATE_PARAMS,
    ),
    write(
        "envelopes.send",
        "send",
        "Send envelope",
        "Send an envelope for signature.",
        GET_PARAMS,
    ),
    destructive(
        "envelopes.void",
        "void",
        "Void envelope",
        "Void an in-progress envelope.",
        DELETE_PARAMS,
    ),
];

pub(crate) const AUTOMATION_ACTIONS: &[ActionTemplate] = &[
    read(
        "workflows.list",
        "list",
        "List workflows",
        "List automation workflows.",
        LIST_PARAMS,
    ),
    read(
        "runs.list",
        "list",
        "List runs",
        "List workflow executions.",
        GET_PARAMS,
    ),
    write(
        "workflows.create",
        "create",
        "Create workflow",
        "Create an automation workflow.",
        CREATE_PARAMS,
    ),
    write(
        "workflows.run",
        "run",
        "Run workflow",
        "Run an automation workflow.",
        WORKFLOW_RUN,
    ),
    destructive(
        "workflows.delete",
        "delete",
        "Delete workflow",
        "Delete an automation workflow.",
        DELETE_PARAMS,
    ),
];

pub(crate) const ADMIN_ACTIONS: &[ActionTemplate] = &[
    read(
        "users.list",
        "list",
        "List users",
        "List organization users.",
        LIST_PARAMS,
    ),
    read(
        "groups.list",
        "list",
        "List groups",
        "List organization groups.",
        LIST_PARAMS,
    ),
    write(
        "users.create",
        "create",
        "Create user",
        "Create or invite a user.",
        CREATE_PARAMS,
    ),
    write(
        "users.update",
        "update",
        "Update user",
        "Update user attributes or membership.",
        UPDATE_PARAMS,
    ),
    destructive(
        "users.suspend",
        "suspend",
        "Suspend user",
        "Suspend a user account.",
        DELETE_PARAMS,
    ),
];

pub(crate) const DESIGN_ACTIONS: &[ActionTemplate] = &[
    read(
        "assets.list",
        "list",
        "List assets",
        "List designs or media assets.",
        LIST_PARAMS,
    ),
    read(
        "assets.search",
        "search",
        "Search assets",
        "Search designs or media assets.",
        SEARCH_PARAMS,
    ),
    write(
        "assets.create",
        "create",
        "Create asset",
        "Create a design or media asset.",
        CREATE_PARAMS,
    ),
    write(
        "assets.export",
        "export",
        "Export asset",
        "Export an asset in a selected format.",
        GET_PARAMS,
    ),
    destructive(
        "assets.delete",
        "delete",
        "Delete asset",
        "Delete a design or media asset.",
        DELETE_PARAMS,
    ),
];

pub(crate) const MEDIA_ACTIONS: &[ActionTemplate] = &[
    read(
        "library.list",
        "list",
        "List library",
        "List media in the user's library.",
        LIST_PARAMS,
    ),
    read(
        "library.search",
        "search",
        "Search media",
        "Search media and creators.",
        SEARCH_PARAMS,
    ),
    read(
        "items.get",
        "get",
        "Get media",
        "Read media metadata.",
        GET_PARAMS,
    ),
    write(
        "playlists.create",
        "create",
        "Create playlist",
        "Create a media playlist.",
        CREATE_PARAMS,
    ),
    destructive(
        "playlists.delete",
        "delete",
        "Delete playlist",
        "Delete a media playlist.",
        DELETE_PARAMS,
    ),
];
