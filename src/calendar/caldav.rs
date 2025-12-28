use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;

use super::CalendarEngine;
use crate::shared::state::AppState;

pub fn create_caldav_router(_engine: Arc<CalendarEngine>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/caldav", get(caldav_root))
        .route("/caldav/principals", get(caldav_principals))
        .route("/caldav/calendars", get(caldav_calendars))
        .route("/caldav/calendars/:calendar_id", get(caldav_calendar))
        .route(
            "/caldav/calendars/:calendar_id/:event_id.ics",
            get(caldav_event).put(caldav_put_event),
        )
}

async fn caldav_root() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("DAV", "1, 2, calendar-access")
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
    <D:response>
        <D:href>/caldav/</D:href>
        <D:propstat>
            <D:prop>
                <D:resourcetype>
                    <D:collection/>
                </D:resourcetype>
                <D:displayname>GeneralBots CalDAV Server</D:displayname>
            </D:prop>
            <D:status>HTTP/1.1 200 OK</D:status>
        </D:propstat>
    </D:response>
</D:multistatus>"#
                .to_string(),
        )
        .expect("valid response")
}

async fn caldav_principals() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
    <D:response>
        <D:href>/caldav/principals/</D:href>
        <D:propstat>
            <D:prop>
                <D:resourcetype>
                    <D:collection/>
                    <D:principal/>
                </D:resourcetype>
                <C:calendar-home-set>
                    <D:href>/caldav/calendars/</D:href>
                </C:calendar-home-set>
            </D:prop>
            <D:status>HTTP/1.1 200 OK</D:status>
        </D:propstat>
    </D:response>
</D:multistatus>"#
                .to_string(),
        )
        .expect("valid response")
}

async fn caldav_calendars() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
    <D:response>
        <D:href>/caldav/calendars/</D:href>
        <D:propstat>
            <D:prop>
                <D:resourcetype>
                    <D:collection/>
                </D:resourcetype>
                <D:displayname>Calendars</D:displayname>
            </D:prop>
            <D:status>HTTP/1.1 200 OK</D:status>
        </D:propstat>
    </D:response>
    <D:response>
        <D:href>/caldav/calendars/default/</D:href>
        <D:propstat>
            <D:prop>
                <D:resourcetype>
                    <D:collection/>
                    <C:calendar/>
                </D:resourcetype>
                <D:displayname>Default Calendar</D:displayname>
                <C:supported-calendar-component-set>
                    <C:comp name="VEVENT"/>
                    <C:comp name="VTODO"/>
                </C:supported-calendar-component-set>
            </D:prop>
            <D:status>HTTP/1.1 200 OK</D:status>
        </D:propstat>
    </D:response>
</D:multistatus>"#
                .to_string(),
        )
        .expect("valid response")
}

async fn caldav_calendar() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
    <D:response>
        <D:href>/caldav/calendars/default/</D:href>
        <D:propstat>
            <D:prop>
                <D:resourcetype>
                    <D:collection/>
                    <C:calendar/>
                </D:resourcetype>
                <D:displayname>Default Calendar</D:displayname>
            </D:prop>
            <D:status>HTTP/1.1 200 OK</D:status>
        </D:propstat>
    </D:response>
</D:multistatus>"#
                .to_string(),
        )
        .expect("valid response")
}

async fn caldav_event() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/calendar; charset=utf-8")
        .body(
            r"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-
BEGIN:VEVENT
UID:placeholder@generalbots.com
DTSTAMP:20240101T000000Z
DTSTART:20240101T090000Z
DTEND:20240101T100000Z
SUMMARY:Placeholder Event
END:VEVENT
END:VCALENDAR"
                .to_string(),
        )
        .expect("valid response")
}

async fn caldav_put_event() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::CREATED)
        .header("ETag", "\"placeholder-etag\"")
        .body(String::new())
        .expect("valid response")
}
