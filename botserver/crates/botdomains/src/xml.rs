//! XML reader for the Namecheap API (issue #1469).
//!
//! Namecheap is the one registrar in the set that answers in XML. Its documents
//! are shallow and regular, so a streaming reader over `quick-xml` — already a
//! workspace dependency — is both smaller and more reliable than string
//! matching: `text()`-style extraction silently returns the wrong leaf whenever
//! an element repeats, which is exactly what a zone listing does.
//!
//! Three Namecheap behaviours are handled here because they are easy to get
//! wrong and produce silently wrong answers:
//!
//! * **Errors arrive with HTTP 200.** The status code says nothing; the
//!   `<Errors>` element is the authority.
//! * **List results use positional keys.** `<DomainCheckResult>` holds
//!   `<Domain1>`, `<Domain2>`, …, so order is the only identity.
//! * **Zone records appear in both `<host>` and `<hostattr>`.** MX and SRV carry
//!   their target in a sibling `<hostattr>` block, and dropping those loses the
//!   customer's mail routing.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::RegistrarError;
use crate::http;

/// One `<Error>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    pub number: String,
    pub message: String,
}

/// A parsed response, in the shape the adapter consumes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Response {
    pub errors: Vec<ApiError>,
    /// Flat leaves in document order, as `(element, text)` pairs. Repeated
    /// elements are all retained.
    pub leaves: Vec<(String, String)>,
    /// Repeated block elements, as `(element, inner XML)` pairs. Used for the
    /// zone's `<host>`/`<hostattr>` entries, whose nesting has to be preserved.
    pub blocks: Vec<(String, String)>,
    /// Nested blocks kept verbatim: `(parent, child, inner XML)`.
    pub nested: Vec<(String, String, String)>,
}

/// Parses a response body.
///
/// A body that is not an API response at all is a decode error rather than an
/// empty success — a gateway HTML page must never read as "no errors".
///
/// Every element's inner XML is captured, not just the outermost one: a zone
/// listing nests its records two levels deep, and capturing only the container
/// would leave the adapter with nothing to read.
pub fn parse(service: &str, body: &str) -> Result<Response, RegistrarError> {
    if !body.contains("<ApiResponse") {
        return Err(RegistrarError::Decode(
            service.into(),
            format!("not an API response: {}", http::truncate(body)),
        ));
    }

    let mut reader = Reader::from_str(body);
    reader.config_mut().trim_text(true);
    reader.config_mut().check_end_names = true;

    let mut response = Response::default();
    // Open elements with the byte offset just past their start tag, which is
    // where their inner XML begins.
    let mut stack: Vec<(String, usize)> = Vec::new();

    loop {
        let event = reader
            .read_event()
            .map_err(|e| RegistrarError::Decode(service.into(), format!("malformed XML: {e}")))?;

        match event {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let start = reader.buffer_position() as usize;
                stack.push((name, start));
            }
            Event::End(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let end = reader.buffer_position() as usize;
                let Some((opened, start)) = stack.pop() else { continue };
                if opened != name {
                    return Err(RegistrarError::Decode(
                        service.into(),
                        format!("mismatched element: <{opened}> closed by </{name}>"),
                    ));
                }
                // `</name>` is three characters wider than the name itself.
                let closing = name.len() + 3;
                let inner_end = end.saturating_sub(closing).max(start);
                let inner = body.get(start..inner_end).unwrap_or("").trim().to_string();

                if is_leaf_interest(&opened) && !inner.is_empty() {
                    response.leaves.push((opened.clone(), inner.clone()));
                }
                let parent = stack.last().map(|(name, _)| name.clone()).unwrap_or_default();
                response.blocks.push((opened.clone(), inner.clone()));
                response.nested.push((parent, opened, inner));
            }
            Event::Eof => break,
            _ => {}
        }
    }

    response.errors = response
        .blocks
        .iter()
        .filter(|(tag, _)| tag == "Error")
        .map(|(_, xml)| ApiError {
            number: leaf_in(xml, "Number").unwrap_or_default(),
            message: leaf_in(xml, "Message").unwrap_or_default(),
        })
        .collect();

    Ok(response)
}

/// Elements whose text the adapter reads directly.
fn is_leaf_interest(name: &str) -> bool {
    matches!(
        name,
        "CommandResponseType"
            | "Domain"
            | "DomainName"
            | "Available"
            | "IsAvailable"
            | "Whois"
            | "GetListResult"
            | "SetHostsResult"
            | "CreateDomainResult"
            | "GetInfoResult"
            | "ExpiredDate"
    )
}


/// First `<tag>value</tag>` in `xml`.
#[must_use]
pub fn leaf_in(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].trim().to_string())
}

/// First value recorded for a leaf element.
#[must_use]
pub fn leaf<'a>(response: &'a Response, tag: &str) -> Option<&'a str> {
    response
        .leaves
        .iter()
        .find(|(name, _)| name == tag)
        .map(|(_, value)| value.as_str())
}

/// The `<host>` and `<hostattr>` blocks of a zone, in document order.
#[must_use]
pub fn zone_blocks(response: &Response) -> Vec<(String, String)> {
    response
        .blocks
        .iter()
        .filter(|(tag, _)| tag == "host" || tag == "hostattr")
        .cloned()
        .collect()
}

/// Every `<DomainN>` availability flag under `<DomainCheckResult>`, in order.
#[must_use]
pub fn availability_flags(response: &Response) -> Vec<String> {
    response
        .nested
        .iter()
        .filter(|(_, child, _)| child.starts_with("Domain") && child != "DomainCheckResult")
        .filter_map(|(_, _, xml)| leaf_in(xml, "Available"))
        .collect()
}

/// Every `<Error>` rendered as `number: message`, joined for error mapping.
#[must_use]
pub fn error_summary(response: &Response) -> Option<String> {
    (!response.errors.is_empty()).then(|| {
        response
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.number, e.message))
            .collect::<Vec<_>>()
            .join("; ")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RegistrarError;

    const OK: &str = r#"<?xml version="1.0"?>
<ApiResponse Status="OK">
  <Errors />
  <CommandResponseType>getList</CommandResponseType>
  <Domain>acme.com</Domain>
  <DomainGetListResult>
    <host><Host>@</Host><Type>A</Type><Address>203.0.113.7</Address><TTL>1800</TTL></host>
    <host><Host>_dmarc</Host><Type>TXT</Type><Address>v=DMARC1; p=reject</Address><TTL>1800</TTL></host>
  </DomainGetListResult>
</ApiResponse>"#;

    const ERRORS: &str = r#"<ApiResponse Status="ERROR"><Errors>
  <Error><Number>20110</Number><Message>Invalid API key.</Message></Error>
  <Error><Number>20118</Number><Message>Too many requests.</Message></Error>
</Errors></ApiResponse>"#;

    #[test]
    fn a_gateway_html_page_is_not_mistaken_for_a_success() {
        let err = parse("Namecheap", "<html>502 Bad Gateway</html>").unwrap_err();
        assert!(matches!(err, RegistrarError::Decode(_, _)));
        assert!(err.to_string().contains("502"));
    }

    #[test]
    fn errors_survive_a_200_response() {
        let parsed = parse("Namecheap", ERRORS).expect("parsed");
        assert_eq!(parsed.errors.len(), 2, "{:?}", parsed.errors);
        assert_eq!(parsed.errors[0].number, "20110");
        let summary = error_summary(&parsed).expect("summary");
        assert!(summary.contains("20110") && summary.contains("20118"));
    }

    #[test]
    fn an_ok_response_reports_no_errors() {
        let parsed = parse("Namecheap", OK).expect("parsed");
        assert!(parsed.errors.is_empty());
        assert!(error_summary(&parsed).is_none());
    }

    #[test]
    fn leaves_are_read() {
        let parsed = parse("Namecheap", OK).expect("parsed");
        assert_eq!(leaf(&parsed, "Domain"), Some("acme.com"));
        assert_eq!(leaf(&parsed, "CommandResponseType"), Some("getList"));
        assert_eq!(leaf(&parsed, "Absent"), None);
    }

    #[test]
    fn every_repeated_zone_record_is_captured_in_order() {
        let parsed = parse("Namecheap", OK).expect("parsed");
        let hosts = zone_blocks(&parsed);
        assert_eq!(hosts.len(), 2, "{hosts:?}");
        assert_eq!(leaf_in(&hosts[0].1, "Host").as_deref(), Some("@"));
        assert_eq!(leaf_in(&hosts[1].1, "Host").as_deref(), Some("_dmarc"));
        assert_eq!(leaf_in(&hosts[0].1, "Address").as_deref(), Some("203.0.113.7"));
    }

    #[test]
    fn mx_hostattr_blocks_are_preserved() {
        // MX records carry their target in a sibling <hostattr>; dropping those
        // would silently lose the customer's mail routing.
        let xml = r#"<ApiResponse Status="OK"><Errors/>
          <DomainGetListResult>
            <host><Host>@</Host><Type>MX</Type><Address>mail1.example.net</Address><TTL>1800</TTL></host>
            <hostattr><Host>@</Host><Type>MX</Type><Value>10</Value><Attribution>mail1.example.net</Attribution></hostattr>
          </DomainGetListResult>
        </ApiResponse>"#;
        let parsed = parse("Namecheap", xml).expect("parsed");
        let hosts = zone_blocks(&parsed);
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[1].0, "hostattr");
        assert_eq!(leaf_in(&hosts[1].1, "Attribution").as_deref(), Some("mail1.example.net"));
    }

    #[test]
    fn a_check_response_yields_flags_in_request_order() {
        let xml = r#"<ApiResponse Status="OK"><Errors/><DomainCheckResult>
          <Domain1><Domain>acme.com</Domain><Available>true</Available></Domain1>
          <Domain2><Domain>taken.com</Domain><Available>false</Available></Domain2>
          <Domain3><Domain>premium.com</Domain><Available>true</Available></Domain3>
        </DomainCheckResult></ApiResponse>"#;
        let parsed = parse("Namecheap", xml).expect("parsed");
        assert_eq!(availability_flags(&parsed), vec!["true", "false", "true"]);
    }

    #[test]
    fn escaped_entities_are_decoded() {
        let xml = r#"<ApiResponse Status="OK"><Errors/><Domain>acme.com</Domain></ApiResponse>"#;
        let parsed = parse("Namecheap", xml).expect("parsed");
        assert_eq!(leaf(&parsed, "Domain"), Some("acme.com"));
    }
}
