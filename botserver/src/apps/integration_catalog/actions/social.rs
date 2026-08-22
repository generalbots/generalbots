use super::super::types::{ActionTemplate, Parameter, ParameterType};
use super::{
    destructive, param, read, write, CREATE_PARAMS, DELETE_PARAMS, GET_PARAMS, LIST_PARAMS,
    SEARCH_PARAMS, UPDATE_PARAMS,
};

const METRIC_RANGE: &[Parameter] = &[
    param(
        "metric",
        ParameterType::String,
        true,
        "Health or activity metric",
    ),
    param("start", ParameterType::DateTime, false, "Range start"),
    param("end", ParameterType::DateTime, false, "Range end"),
];
const DEVICE_COMMAND: &[Parameter] = &[
    param(
        "device_id",
        ParameterType::String,
        true,
        "Device identifier",
    ),
    param("state", ParameterType::Json, true, "Requested device state"),
];

pub(crate) const SOCIAL_ACTIONS: &[ActionTemplate] = &[
    read(
        "posts.list",
        "list",
        "List posts",
        "List social posts.",
        LIST_PARAMS,
    ),
    read(
        "posts.search",
        "search",
        "Search posts",
        "Search social posts.",
        SEARCH_PARAMS,
    ),
    read(
        "posts.get",
        "get",
        "Get post",
        "Read a social post and engagement.",
        GET_PARAMS,
    ),
    write(
        "posts.create",
        "create",
        "Create post",
        "Publish a social post.",
        CREATE_PARAMS,
    ),
    destructive(
        "posts.delete",
        "delete",
        "Delete post",
        "Delete a social post.",
        DELETE_PARAMS,
    ),
];

pub(crate) const CONTENT_ACTIONS: &[ActionTemplate] = &[
    read(
        "content.list",
        "list",
        "List content",
        "List published and draft content.",
        LIST_PARAMS,
    ),
    read(
        "content.search",
        "search",
        "Search content",
        "Search published and draft content.",
        SEARCH_PARAMS,
    ),
    read(
        "content.get",
        "get",
        "Get content",
        "Read content and metadata.",
        GET_PARAMS,
    ),
    write(
        "content.create",
        "create",
        "Create content",
        "Create a content draft.",
        CREATE_PARAMS,
    ),
    write(
        "content.update",
        "update",
        "Update content",
        "Update content.",
        UPDATE_PARAMS,
    ),
    destructive(
        "content.delete",
        "delete",
        "Delete content",
        "Delete content.",
        DELETE_PARAMS,
    ),
];

pub(crate) const BOOKMARK_ACTIONS: &[ActionTemplate] = &[
    read(
        "bookmarks.list",
        "list",
        "List bookmarks",
        "List saved bookmarks or highlights.",
        LIST_PARAMS,
    ),
    read(
        "bookmarks.search",
        "search",
        "Search bookmarks",
        "Search saved bookmarks or highlights.",
        SEARCH_PARAMS,
    ),
    read(
        "bookmarks.get",
        "get",
        "Get bookmark",
        "Read a bookmark or highlight.",
        GET_PARAMS,
    ),
    write(
        "bookmarks.create",
        "create",
        "Create bookmark",
        "Save a bookmark or highlight.",
        CREATE_PARAMS,
    ),
    destructive(
        "bookmarks.delete",
        "delete",
        "Delete bookmark",
        "Delete a bookmark or highlight.",
        DELETE_PARAMS,
    ),
];

pub(crate) const HEALTH_ACTIONS: &[ActionTemplate] = &[
    read(
        "metrics.list",
        "list",
        "List metrics",
        "List available health and activity metrics.",
        LIST_PARAMS,
    ),
    read(
        "metrics.query",
        "query",
        "Query metrics",
        "Query health or activity metrics over time.",
        METRIC_RANGE,
    ),
    read(
        "activities.list",
        "list",
        "List activities",
        "List workouts, sleep, or activity records.",
        LIST_PARAMS,
    ),
    write(
        "goals.update",
        "update",
        "Update goal",
        "Update a health or activity goal.",
        UPDATE_PARAMS,
    ),
    destructive(
        "activities.delete",
        "delete",
        "Delete activity",
        "Delete an activity record.",
        DELETE_PARAMS,
    ),
];

pub(crate) const SMART_HOME_ACTIONS: &[ActionTemplate] = &[
    read(
        "devices.list",
        "list",
        "List devices",
        "List smart-home devices.",
        LIST_PARAMS,
    ),
    read(
        "devices.get",
        "get",
        "Get device",
        "Read device state and capabilities.",
        GET_PARAMS,
    ),
    read(
        "scenes.list",
        "list",
        "List scenes",
        "List smart-home scenes.",
        LIST_PARAMS,
    ),
    write(
        "devices.update",
        "update",
        "Update device",
        "Change a smart-home device state.",
        DEVICE_COMMAND,
    ),
    write(
        "scenes.activate",
        "activate",
        "Activate scene",
        "Activate a smart-home scene.",
        GET_PARAMS,
    ),
];

pub(crate) const NEWS_ACTIONS: &[ActionTemplate] = &[
    read(
        "articles.search",
        "search",
        "Search articles",
        "Search news articles.",
        SEARCH_PARAMS,
    ),
    read(
        "sources.list",
        "list",
        "List sources",
        "List news sources.",
        LIST_PARAMS,
    ),
    read(
        "articles.get",
        "get",
        "Get article",
        "Read article metadata and available text.",
        GET_PARAMS,
    ),
    write(
        "alerts.create",
        "create",
        "Create alert",
        "Create a news search alert.",
        CREATE_PARAMS,
    ),
    destructive(
        "alerts.delete",
        "delete",
        "Delete alert",
        "Delete a news search alert.",
        DELETE_PARAMS,
    ),
];
