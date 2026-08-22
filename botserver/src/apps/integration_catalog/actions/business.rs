use super::super::types::{ActionTemplate, Parameter, ParameterType};
use super::{
    destructive, param, read, write, CREATE_PARAMS, DELETE_PARAMS, GET_PARAMS, LIST_PARAMS,
    SEARCH_PARAMS, UPDATE_PARAMS,
};

const CAMPAIGN_SEND: &[Parameter] = &[param(
    "campaign_id",
    ParameterType::String,
    true,
    "Campaign identifier",
)];
const REPORT_RANGE: &[Parameter] = &[
    param("start", ParameterType::DateTime, false, "Range start"),
    param("end", ParameterType::DateTime, false, "Range end"),
];

pub(crate) const CRM_ACTIONS: &[ActionTemplate] = &[
    read(
        "contacts.list",
        "list",
        "List contacts",
        "List CRM contacts.",
        LIST_PARAMS,
    ),
    read(
        "contacts.search",
        "search",
        "Search contacts",
        "Search CRM contacts.",
        SEARCH_PARAMS,
    ),
    read(
        "deals.list",
        "list",
        "List deals",
        "List sales deals or opportunities.",
        LIST_PARAMS,
    ),
    write(
        "contacts.create",
        "create",
        "Create contact",
        "Create a CRM contact.",
        CREATE_PARAMS,
    ),
    write(
        "contacts.update",
        "update",
        "Update contact",
        "Update a CRM contact.",
        UPDATE_PARAMS,
    ),
    destructive(
        "contacts.delete",
        "delete",
        "Delete contact",
        "Delete a CRM contact.",
        DELETE_PARAMS,
    ),
];

pub(crate) const MARKETING_ACTIONS: &[ActionTemplate] = &[
    read(
        "campaigns.list",
        "list",
        "List campaigns",
        "List marketing campaigns.",
        LIST_PARAMS,
    ),
    read(
        "contacts.search",
        "search",
        "Search contacts",
        "Search marketing contacts or subscribers.",
        SEARCH_PARAMS,
    ),
    read(
        "campaigns.reports.get",
        "get",
        "Get campaign report",
        "Read campaign performance.",
        GET_PARAMS,
    ),
    write(
        "campaigns.create",
        "create",
        "Create campaign",
        "Create a marketing campaign.",
        CREATE_PARAMS,
    ),
    write(
        "campaigns.send",
        "send",
        "Send campaign",
        "Send a marketing campaign.",
        CAMPAIGN_SEND,
    ),
    destructive(
        "campaigns.delete",
        "delete",
        "Delete campaign",
        "Delete a marketing campaign.",
        DELETE_PARAMS,
    ),
];

pub(crate) const SEO_ACTIONS: &[ActionTemplate] = &[
    read(
        "sites.list",
        "list",
        "List sites",
        "List tracked sites.",
        LIST_PARAMS,
    ),
    read(
        "keywords.search",
        "search",
        "Search keywords",
        "Research keyword performance.",
        SEARCH_PARAMS,
    ),
    read(
        "reports.get",
        "get",
        "Get SEO report",
        "Read an SEO report.",
        GET_PARAMS,
    ),
    write(
        "crawls.start",
        "start",
        "Start crawl",
        "Start a site crawl.",
        GET_PARAMS,
    ),
    destructive(
        "projects.delete",
        "delete",
        "Delete project",
        "Delete an SEO project.",
        DELETE_PARAMS,
    ),
];

pub(crate) const FINANCE_OPERATIONS_ACTIONS: &[ActionTemplate] = &[
    read(
        "transactions.list",
        "list",
        "List transactions",
        "List financial transactions.",
        REPORT_RANGE,
    ),
    read(
        "transactions.search",
        "search",
        "Search transactions",
        "Search financial transactions.",
        SEARCH_PARAMS,
    ),
    read(
        "expenses.get",
        "get",
        "Get expense",
        "Read expense details.",
        GET_PARAMS,
    ),
    write(
        "expenses.create",
        "create",
        "Create expense",
        "Create an expense record.",
        CREATE_PARAMS,
    ),
    write(
        "expenses.update",
        "update",
        "Update expense",
        "Update an expense record.",
        UPDATE_PARAMS,
    ),
    destructive(
        "expenses.delete",
        "delete",
        "Delete expense",
        "Delete an expense record.",
        DELETE_PARAMS,
    ),
];

pub(crate) const HR_ACTIONS: &[ActionTemplate] = &[
    read(
        "employees.list",
        "list",
        "List employees",
        "List employees or candidates.",
        LIST_PARAMS,
    ),
    read(
        "employees.search",
        "search",
        "Search employees",
        "Search employees or candidates.",
        SEARCH_PARAMS,
    ),
    read(
        "employees.get",
        "get",
        "Get employee",
        "Read employee or candidate details.",
        GET_PARAMS,
    ),
    write(
        "employees.create",
        "create",
        "Create employee",
        "Create an employee or candidate record.",
        CREATE_PARAMS,
    ),
    write(
        "employees.update",
        "update",
        "Update employee",
        "Update an employee or candidate record.",
        UPDATE_PARAMS,
    ),
    destructive(
        "employees.terminate",
        "terminate",
        "Terminate employee",
        "Terminate or archive an employee record.",
        DELETE_PARAMS,
    ),
];

pub(crate) const SUPPORT_ACTIONS: &[ActionTemplate] = &[
    read(
        "tickets.list",
        "list",
        "List tickets",
        "List support tickets.",
        LIST_PARAMS,
    ),
    read(
        "tickets.search",
        "search",
        "Search tickets",
        "Search support tickets.",
        SEARCH_PARAMS,
    ),
    read(
        "tickets.get",
        "get",
        "Get ticket",
        "Read a support ticket and conversation.",
        GET_PARAMS,
    ),
    write(
        "tickets.create",
        "create",
        "Create ticket",
        "Create a support ticket.",
        CREATE_PARAMS,
    ),
    write(
        "tickets.update",
        "update",
        "Update ticket",
        "Update a support ticket.",
        UPDATE_PARAMS,
    ),
    destructive(
        "tickets.delete",
        "delete",
        "Delete ticket",
        "Delete a support ticket.",
        DELETE_PARAMS,
    ),
];

pub(crate) const COMMERCE_ACTIONS: &[ActionTemplate] = &[
    read(
        "products.list",
        "list",
        "List products",
        "List commerce products.",
        LIST_PARAMS,
    ),
    read(
        "products.search",
        "search",
        "Search products",
        "Search commerce products.",
        SEARCH_PARAMS,
    ),
    read(
        "orders.list",
        "list",
        "List orders",
        "List commerce orders.",
        LIST_PARAMS,
    ),
    write(
        "products.create",
        "create",
        "Create product",
        "Create a commerce product.",
        CREATE_PARAMS,
    ),
    write(
        "products.update",
        "update",
        "Update product",
        "Update a commerce product.",
        UPDATE_PARAMS,
    ),
    destructive(
        "products.delete",
        "delete",
        "Delete product",
        "Delete a commerce product.",
        DELETE_PARAMS,
    ),
];

pub(crate) const ACCOUNTING_ACTIONS: &[ActionTemplate] = &[
    read(
        "invoices.list",
        "list",
        "List invoices",
        "List accounting invoices.",
        LIST_PARAMS,
    ),
    read(
        "invoices.search",
        "search",
        "Search invoices",
        "Search accounting invoices.",
        SEARCH_PARAMS,
    ),
    read(
        "reports.get",
        "get",
        "Get report",
        "Read an accounting report.",
        GET_PARAMS,
    ),
    write(
        "invoices.create",
        "create",
        "Create invoice",
        "Create an accounting invoice.",
        CREATE_PARAMS,
    ),
    write(
        "invoices.update",
        "update",
        "Update invoice",
        "Update an accounting invoice.",
        UPDATE_PARAMS,
    ),
    destructive(
        "invoices.void",
        "void",
        "Void invoice",
        "Void an accounting invoice.",
        DELETE_PARAMS,
    ),
];
