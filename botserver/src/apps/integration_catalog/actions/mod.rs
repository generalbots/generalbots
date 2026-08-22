mod aws;
mod business;
mod collaboration;
mod developer;
mod office;
mod social;
mod transactions;

pub(crate) use aws::*;
pub(crate) use business::*;
pub(crate) use collaboration::*;
pub(crate) use developer::*;
pub(crate) use office::*;
pub(crate) use social::*;
pub(crate) use transactions::*;

use super::types::{ActionTemplate, Parameter, ParameterType, Risk, Surface};

pub(crate) const ALL_SURFACES: &[Surface] = &[Surface::Chat, Surface::Ui, Surface::Api];

pub(crate) const fn param(
    name: &'static str,
    parameter_type: ParameterType,
    required: bool,
    description: &'static str,
) -> Parameter {
    Parameter {
        name,
        parameter_type,
        required,
        description,
    }
}

pub(crate) const LIST_PARAMS: &[Parameter] = &[param(
    "limit",
    ParameterType::Integer,
    false,
    "Maximum number of results",
)];
pub(crate) const SEARCH_PARAMS: &[Parameter] = &[
    param(
        "query",
        ParameterType::String,
        true,
        "Provider search query",
    ),
    param(
        "limit",
        ParameterType::Integer,
        false,
        "Maximum number of results",
    ),
];
pub(crate) const GET_PARAMS: &[Parameter] = &[param(
    "resource_id",
    ParameterType::String,
    true,
    "Provider resource identifier",
)];
pub(crate) const CREATE_PARAMS: &[Parameter] = &[param(
    "data",
    ParameterType::Json,
    true,
    "Provider-specific fields for the new resource",
)];
pub(crate) const UPDATE_PARAMS: &[Parameter] = &[
    param(
        "resource_id",
        ParameterType::String,
        true,
        "Provider resource identifier",
    ),
    param("changes", ParameterType::Json, true, "Fields to update"),
];
pub(crate) const DELETE_PARAMS: &[Parameter] = &[param(
    "resource_id",
    ParameterType::String,
    true,
    "Provider resource identifier to delete",
)];

pub(crate) const fn read(
    key: &'static str,
    verb: &'static str,
    label: &'static str,
    summary: &'static str,
    params: &'static [Parameter],
) -> ActionTemplate {
    ActionTemplate {
        key,
        verb,
        label,
        summary,
        params,
        risk: Risk::Low,
        requires_approval: false,
        surfaces: ALL_SURFACES,
    }
}

pub(crate) const fn write(
    key: &'static str,
    verb: &'static str,
    label: &'static str,
    summary: &'static str,
    params: &'static [Parameter],
) -> ActionTemplate {
    ActionTemplate {
        key,
        verb,
        label,
        summary,
        params,
        risk: Risk::Medium,
        requires_approval: true,
        surfaces: ALL_SURFACES,
    }
}

pub(crate) const fn destructive(
    key: &'static str,
    verb: &'static str,
    label: &'static str,
    summary: &'static str,
    params: &'static [Parameter],
) -> ActionTemplate {
    ActionTemplate {
        key,
        verb,
        label,
        summary,
        params,
        risk: Risk::High,
        requires_approval: true,
        surfaces: ALL_SURFACES,
    }
}
