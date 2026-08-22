use super::super::types::{ActionTemplate, Parameter, ParameterType};
use super::{
    destructive, param, read, write, CREATE_PARAMS, DELETE_PARAMS, GET_PARAMS, LIST_PARAMS,
    SEARCH_PARAMS, UPDATE_PARAMS,
};

const REPOSITORY_GET: &[Parameter] = &[param(
    "repository",
    ParameterType::String,
    true,
    "Repository owner and name",
)];
const ISSUE_SEARCH: &[Parameter] = &[
    param(
        "repository",
        ParameterType::String,
        false,
        "Repository owner and name",
    ),
    param("query", ParameterType::String, true, "Issue search query"),
];
const METRIC_QUERY: &[Parameter] = &[
    param(
        "metric",
        ParameterType::String,
        true,
        "Metric name or expression",
    ),
    param("start", ParameterType::DateTime, false, "Range start"),
    param("end", ParameterType::DateTime, false, "Range end"),
];
const EMAIL_SEND: &[Parameter] = &[
    param("to", ParameterType::String, true, "Recipient address"),
    param("subject", ParameterType::String, true, "Message subject"),
    param("body", ParameterType::String, true, "Message body"),
];
const INDEX_QUERY: &[Parameter] = &[
    param("index", ParameterType::String, true, "Search index"),
    param("query", ParameterType::String, true, "Search query"),
    param(
        "limit",
        ParameterType::Integer,
        false,
        "Maximum number of hits",
    ),
];
const MODEL_RUN: &[Parameter] = &[
    param("model", ParameterType::String, true, "Model identifier"),
    param("input", ParameterType::Json, true, "Model input"),
];

pub(crate) const REPOSITORY_ACTIONS: &[ActionTemplate] = &[
    read(
        "repositories.list",
        "list",
        "List repositories",
        "List accessible repositories.",
        LIST_PARAMS,
    ),
    read(
        "repositories.get",
        "get",
        "Get repository",
        "Read repository metadata.",
        REPOSITORY_GET,
    ),
    read(
        "issues.search",
        "search",
        "Search issues",
        "Search repository issues.",
        ISSUE_SEARCH,
    ),
    read(
        "pull_requests.list",
        "list",
        "List pull requests",
        "List repository pull requests.",
        REPOSITORY_GET,
    ),
    write(
        "issues.create",
        "create",
        "Create issue",
        "Create a repository issue.",
        CREATE_PARAMS,
    ),
    write(
        "issues.update",
        "update",
        "Update issue",
        "Update a repository issue.",
        UPDATE_PARAMS,
    ),
];

pub(crate) const CLOUD_ACTIONS: &[ActionTemplate] = &[
    read(
        "resources.list",
        "list",
        "List resources",
        "List cloud resources.",
        LIST_PARAMS,
    ),
    read(
        "resources.search",
        "search",
        "Search resources",
        "Search cloud resources.",
        SEARCH_PARAMS,
    ),
    read(
        "resources.get",
        "get",
        "Get resource",
        "Read cloud resource details.",
        GET_PARAMS,
    ),
    write(
        "resources.create",
        "create",
        "Create resource",
        "Create a cloud resource.",
        CREATE_PARAMS,
    ),
    destructive(
        "resources.delete",
        "delete",
        "Delete resource",
        "Delete a cloud resource.",
        DELETE_PARAMS,
    ),
];

pub(crate) const DEPLOYMENT_ACTIONS: &[ActionTemplate] = &[
    read(
        "projects.list",
        "list",
        "List projects",
        "List deployment projects.",
        LIST_PARAMS,
    ),
    read(
        "deployments.list",
        "list",
        "List deployments",
        "List project deployments.",
        GET_PARAMS,
    ),
    read(
        "deployments.get",
        "get",
        "Get deployment",
        "Read deployment status and logs.",
        GET_PARAMS,
    ),
    write(
        "deployments.create",
        "create",
        "Create deployment",
        "Start a project deployment.",
        CREATE_PARAMS,
    ),
    destructive(
        "deployments.cancel",
        "cancel",
        "Cancel deployment",
        "Cancel an active deployment.",
        DELETE_PARAMS,
    ),
];

pub(crate) const DATA_PLATFORM_ACTIONS: &[ActionTemplate] = &[
    read(
        "datasets.list",
        "list",
        "List datasets",
        "List datasets, tables, or warehouses.",
        LIST_PARAMS,
    ),
    read(
        "datasets.search",
        "search",
        "Search datasets",
        "Search data assets.",
        SEARCH_PARAMS,
    ),
    read(
        "queries.get",
        "get",
        "Get query",
        "Read a saved query.",
        GET_PARAMS,
    ),
    write(
        "queries.run",
        "run",
        "Run query",
        "Run a read-only data query.",
        CREATE_PARAMS,
    ),
    write(
        "jobs.create",
        "create",
        "Create job",
        "Create a data processing job.",
        CREATE_PARAMS,
    ),
];

pub(crate) const ANALYTICS_ACTIONS: &[ActionTemplate] = &[
    read(
        "events.list",
        "list",
        "List events",
        "List analytics events.",
        LIST_PARAMS,
    ),
    read(
        "events.search",
        "search",
        "Search events",
        "Search analytics events.",
        SEARCH_PARAMS,
    ),
    read(
        "metrics.query",
        "query",
        "Query metrics",
        "Query product or audience metrics.",
        METRIC_QUERY,
    ),
    write(
        "annotations.create",
        "create",
        "Create annotation",
        "Create an analytics annotation.",
        CREATE_PARAMS,
    ),
    destructive(
        "annotations.delete",
        "delete",
        "Delete annotation",
        "Delete an analytics annotation.",
        DELETE_PARAMS,
    ),
];

pub(crate) const OBSERVABILITY_ACTIONS: &[ActionTemplate] = &[
    read(
        "issues.list",
        "list",
        "List issues",
        "List errors, incidents, or alerts.",
        LIST_PARAMS,
    ),
    read(
        "logs.search",
        "search",
        "Search logs",
        "Search observability logs.",
        SEARCH_PARAMS,
    ),
    read(
        "metrics.query",
        "query",
        "Query metrics",
        "Query observability metrics.",
        METRIC_QUERY,
    ),
    write(
        "alerts.create",
        "create",
        "Create alert",
        "Create an alert rule.",
        CREATE_PARAMS,
    ),
    destructive(
        "alerts.delete",
        "delete",
        "Delete alert",
        "Delete an alert rule.",
        DELETE_PARAMS,
    ),
];

pub(crate) const EMAIL_DELIVERY_ACTIONS: &[ActionTemplate] = &[
    read(
        "messages.list",
        "list",
        "List messages",
        "List delivered email messages.",
        LIST_PARAMS,
    ),
    read(
        "messages.get",
        "get",
        "Get message",
        "Read email delivery details.",
        GET_PARAMS,
    ),
    read(
        "domains.list",
        "list",
        "List domains",
        "List sending domains.",
        LIST_PARAMS,
    ),
    write(
        "messages.send",
        "send",
        "Send email",
        "Send a transactional email.",
        EMAIL_SEND,
    ),
    destructive(
        "domains.delete",
        "delete",
        "Delete domain",
        "Delete a sending domain.",
        DELETE_PARAMS,
    ),
];

pub(crate) const SEARCH_ACTIONS: &[ActionTemplate] = &[
    read(
        "indexes.list",
        "list",
        "List indexes",
        "List search indexes.",
        LIST_PARAMS,
    ),
    read(
        "records.search",
        "search",
        "Search records",
        "Search records in an index.",
        INDEX_QUERY,
    ),
    read(
        "records.get",
        "get",
        "Get record",
        "Read a search record.",
        GET_PARAMS,
    ),
    write(
        "records.upsert",
        "upsert",
        "Upsert record",
        "Create or update a search record.",
        CREATE_PARAMS,
    ),
    destructive(
        "records.delete",
        "delete",
        "Delete record",
        "Delete a search record.",
        DELETE_PARAMS,
    ),
];

pub(crate) const AI_PLATFORM_ACTIONS: &[ActionTemplate] = &[
    read(
        "models.list",
        "list",
        "List models",
        "List available models or agents.",
        LIST_PARAMS,
    ),
    read(
        "models.search",
        "search",
        "Search models",
        "Search models or agents.",
        SEARCH_PARAMS,
    ),
    read(
        "runs.get",
        "get",
        "Get run",
        "Read an inference or agent run.",
        GET_PARAMS,
    ),
    write(
        "models.run",
        "run",
        "Run model",
        "Run a model or agent.",
        MODEL_RUN,
    ),
    destructive(
        "runs.cancel",
        "cancel",
        "Cancel run",
        "Cancel an active model or agent run.",
        DELETE_PARAMS,
    ),
];
