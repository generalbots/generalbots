//! Microsoft 365 integration: SharePoint connector, OAuth2 with Zitadel,
//! Microsoft Graph client interface.
//!
//! The actual HTTP calls live behind the [`graph::GraphClient`] trait so
//! that production can be wired against `reqwest`/`ureq` while unit
//! tests use a stub.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod oauth;
pub mod graph;
pub mod sharepoint;
pub mod connector;

pub use oauth::{M365OAuthFlow, M365Token, M365Scope, M365Error};
pub use graph::{GraphClient, GraphError, GraphSite, GraphList, GraphDriveItem};
pub use sharepoint::{SharePointSite, SharePointList, SharePointListItem, ListColumn};
pub use connector::{M365Source, M365SourceKind, M365SourceConfig, ConnectorError};
