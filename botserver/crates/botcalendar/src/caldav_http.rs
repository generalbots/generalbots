//! HTTP response construction for the CalDAV surface (issue #1335).
//!
//! Every DAV response is built here so the status codes, the headers a client
//! depends on (`DAV`, `Allow`, `WWW-Authenticate`) and the failure bodies stay
//! consistent between the resources. The handlers and the queries live in
//! [`crate::caldav`] and [`crate::caldav_store`].

use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Class levels advertised to clients. `calendar-access` is what makes a client
/// treat the collection as a calendar rather than plain WebDAV.
pub(crate) const DAV_HEADER: &str = "1, 2, 3, calendar-access";

/// Methods every DAV resource answers.
pub(crate) const ALLOW_HEADER: &str = "OPTIONS, GET, HEAD, PUT, DELETE, PROPFIND, PROPPATCH, REPORT";

/// Realm shown in the credential prompt of a calendar client.
const CALENDAR_REALM: &str = "GeneralBots Calendar";

pub(crate) fn options_response() -> Response {
    build_response(
        StatusCode::NO_CONTENT,
        &[
            ("DAV", DAV_HEADER),
            ("Allow", ALLOW_HEADER),
            ("MS-Author-Via", "DAV"),
        ],
        Body::empty(),
    )
}

pub(crate) fn multi_status(xml: String) -> Response {
    build_response(
        StatusCode::MULTI_STATUS,
        &[
            ("Content-Type", "application/xml; charset=utf-8"),
            ("DAV", DAV_HEADER),
        ],
        Body::from(xml),
    )
}

/// Answers an unauthenticated request with the challenge that makes a calendar
/// client prompt for credentials.
pub(crate) fn challenge() -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("DAV", DAV_HEADER)
        .header(
            "WWW-Authenticate",
            format!("Basic realm=\"{CALENDAR_REALM}\", charset=\"UTF-8\""),
        )
        .body(Body::from("Authentication required."))
        .unwrap_or_else(|_| internal_error())
}

pub(crate) fn method_not_allowed() -> Response {
    build_response(
        StatusCode::METHOD_NOT_ALLOWED,
        &[("Allow", ALLOW_HEADER)],
        Body::empty(),
    )
}

pub(crate) fn not_found(message: &str) -> Response {
    build_response(
        StatusCode::NOT_FOUND,
        &[("Content-Type", "text/plain; charset=utf-8")],
        Body::from(message.to_string()),
    )
}

pub(crate) fn forbidden(message: &str) -> Response {
    build_response(
        StatusCode::FORBIDDEN,
        &[("Content-Type", "text/plain; charset=utf-8")],
        Body::from(message.to_string()),
    )
}

/// Logs the underlying failure and answers with an unavailable status, so a
/// database problem is not reported to the client as a protocol error.
pub(crate) fn unavailable(message: String) -> Response {
    log::error!("CalDAV request failed: {message}");
    build_response(
        StatusCode::SERVICE_UNAVAILABLE,
        &[("Content-Type", "text/plain; charset=utf-8")],
        Body::from("The calendar store is unavailable."),
    )
}

/// Builds a response from a status, header pairs and body.
///
/// The builder rejects a malformed header value, which cannot happen for the
/// fixed headers used here; the fallback keeps that impossible case from
/// panicking in a production path.
pub(crate) fn build_response(
    status: StatusCode,
    headers: &[(&'static str, &'static str)],
    body: Body,
) -> Response {
    let mut builder = Response::builder().status(status);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    builder.body(body).unwrap_or_else(|_| internal_error())
}

fn internal_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
}
