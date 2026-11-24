//! Unit tests for the Office deep-link contract (#2475).

use super::{DeepLink, DeepLinkError, OfficeParam, OfficeSurface};

#[test]
fn every_surface_has_a_stable_app_id() {
    let cases = [
        (OfficeSurface::Docs, "docs", "Docs"),
        (OfficeSurface::Sheet, "sheet", "Sheets"),
        (OfficeSurface::Slides, "slides", "Slides"),
    ];
    for (surface, app_id, title) in cases {
        assert_eq!(surface.app_id(), app_id);
        assert_eq!(surface.to_string(), title);
        assert_eq!(OfficeSurface::from_app_id(app_id), Some(surface));
        assert!(OfficeSurface::all().contains(&surface));
    }
    // The registry's plural alias is accepted for Sheets, nothing else is.
    assert_eq!(OfficeSurface::from_app_id("sheets"), Some(OfficeSurface::Sheet));
}

#[test]
fn every_surface_builds_and_parses_without_params() {
    for surface in OfficeSurface::all() {
        let link = DeepLink::new(*surface);
        let rendered = link.render();
        assert_eq!(rendered, format!("app://{}", surface.app_id()));
        assert_eq!(DeepLink::parse(&rendered), Ok(link));
    }
}

#[test]
fn each_surface_accepts_a_file_link() {
    let docs = DeepLink::new(OfficeSurface::Docs)
        .file("reports/Q3.docx")
        .expect("valid file");
    assert_eq!(docs.render(), "app://docs?file=reports%2FQ3.docx");
    assert_eq!(docs.get(OfficeParam::File), Some("reports/Q3.docx"));

    let sheet = DeepLink::new(OfficeSurface::Sheet)
        .file("finance/Q3.xlsx")
        .expect("valid file");
    assert_eq!(sheet.render(), "app://sheet?file=finance%2FQ3.xlsx");

    let slides = DeepLink::new(OfficeSurface::Slides)
        .file(" decks/Q3.pptx ")
        .expect("valid file");
    assert_eq!(slides.render(), "app://slides?file=%20decks%2FQ3.pptx%20");
}

#[test]
fn round_trip_is_deterministic_for_every_surface() {
    for surface in OfficeSurface::all() {
        let mut link = DeepLink::new(*surface);
        link = link.file("drive/relatório 2026.docx").expect("file");
        link = link.vm_conv_id("conv-42").expect("conv id");
        link = link.blank(true).expect("blank");
        if surface.accepts(OfficeParam::VmPath) {
            link = link.vm_path("vm/src/import.csv").expect("vm path");
        }
        let rendered = link.render();
        let parsed = DeepLink::parse(&rendered).expect("parse own output");
        assert_eq!(&parsed, &link, "parse(render(link)) must be the identity");
        assert_eq!(parsed.render(), rendered, "render must be a fixed point");
        assert_eq!(parsed.surface(), *surface);
        assert_eq!(parsed.len(), link.len());
    }
}

#[test]
fn parameter_order_does_not_change_the_link() {
    let first = DeepLink::new(OfficeSurface::Docs)
        .blank(true)
        .expect("blank")
        .file("a.docx")
        .expect("file")
        .vm_conv_id("c-1")
        .expect("conv id");
    let second = DeepLink::new(OfficeSurface::Docs)
        .vm_conv_id("c-1")
        .expect("conv id")
        .file("a.docx")
        .expect("file")
        .blank(true)
        .expect("blank");
    assert_eq!(first.render(), second.render());
    assert_eq!(
        first.render(),
        "app://docs?file=a.docx&vmConvId=c-1&blank=true"
    );
}

#[test]
fn blank_flag_is_a_strict_boolean() {
    let on = DeepLink::new(OfficeSurface::Slides)
        .blank(true)
        .expect("true");
    let off = DeepLink::new(OfficeSurface::Slides)
        .blank(false)
        .expect("false");
    assert_eq!(on.render(), "app://slides?blank=true");
    assert_eq!(off.render(), "app://slides?blank=false");
    assert_eq!(
        DeepLink::new(OfficeSurface::Docs).with_param(OfficeParam::Blank, "maybe"),
        Err(DeepLinkError::InvalidValue {
            param: OfficeParam::Blank,
            value: "maybe".to_string(),
            reason: "blank must be true or false",
        })
    );
}

#[test]
fn unknown_apps_are_refused() {
    assert_eq!(OfficeSurface::from_app_id("mail"), None);
    assert_eq!(
        DeepLink::parse("app://mail?file=x.docx"),
        Err(DeepLinkError::UnknownApp("mail".to_string()))
    );
    assert_eq!(
        DeepLink::parse("app://Docs?file=x.docx"),
        Err(DeepLinkError::UnknownApp("Docs".to_string()))
    );
    assert!(matches!(
        DeepLink::parse("javascript:alert(1)"),
        Err(DeepLinkError::Malformed(_))
    ));
    assert!(matches!(
        DeepLink::parse("app://docs?"),
        Err(DeepLinkError::Malformed(_))
    ));
}

#[test]
fn unknown_and_unsupported_params_are_refused() {
    assert_eq!(
        DeepLink::parse("app://docs?password=secret"),
        Err(DeepLinkError::UnknownParam("password".to_string()))
    );
    assert_eq!(
        DeepLink::parse("app://docs?file=a.docx&file=b.docx"),
        Err(DeepLinkError::DuplicateParam(OfficeParam::File))
    );
    // vmPath is not a Slides capability.
    assert_eq!(
        DeepLink::new(OfficeSurface::Slides).vm_path("vm/src/import.csv"),
        Err(DeepLinkError::UnsupportedParam {
            surface: OfficeSurface::Slides,
            param: OfficeParam::VmPath,
        })
    );
}

#[test]
fn script_schemes_are_refused_even_when_percent_encoded() {
    for hostile in [
        "javascript:alert(1)",
        "JavaScript:alert(1)",
        "java script:alert(1)",
        "  javascript:alert(1)",
        "data:text/html,<script>alert(1)</script>",
        "vbscript:msgbox(1)",
    ] {
        assert!(
            matches!(
                DeepLink::new(OfficeSurface::Docs).file(hostile),
                Err(DeepLinkError::InvalidValue { .. })
            ),
            "builder accepted {hostile:?}"
        );
        let encoded = hostile.replace(' ', "%20").replace(':', "%3A");
        let input = format!("app://docs?file={encoded}");
        assert!(
            matches!(
                DeepLink::parse(&input),
                Err(DeepLinkError::InvalidValue { .. })
            ),
            "parser accepted {input}"
        );
    }
}

#[test]
fn path_traversal_is_refused_for_path_like_params() {
    for hostile in [
        "../etc/passwd",
        "a/../../secret",
        "..\\..\\secret",
        "a/./../b",
        "/root/../../etc/shadow",
    ] {
        assert!(
            matches!(
                DeepLink::new(OfficeSurface::Docs).file(hostile),
                Err(DeepLinkError::InvalidValue { .. })
            ),
            "builder accepted file={hostile:?}"
        );
        assert!(
            matches!(
                DeepLink::new(OfficeSurface::Sheet).vm_path(hostile),
                Err(DeepLinkError::InvalidValue { .. })
            ),
            "builder accepted vmPath={hostile:?}"
        );
    }
    // Encoded separators must not smuggle a `..` segment past the parser.
    for encoded in ["..%2F..%2Fetc", "..%5C..%5Csecret", "%2e%2e%2fsecret"] {
        let input = format!("app://docs?file={encoded}");
        assert!(
            matches!(
                DeepLink::parse(&input),
                Err(DeepLinkError::InvalidValue { .. })
            ),
            "parser accepted {input}"
        );
    }
    // A benign double dot inside a filename is not traversal.
    assert!(DeepLink::new(OfficeSurface::Docs)
        .file("report..v2.docx")
        .is_ok());
}

#[test]
fn control_characters_and_bad_encoding_are_refused() {
    assert!(matches!(
        DeepLink::new(OfficeSurface::Docs).file("a\u{0000}b.docx"),
        Err(DeepLinkError::InvalidValue { .. })
    ));
    for encoded in ["%", "%GG", "%2", "%FF%FE"] {
        let input = format!("app://docs?file={encoded}");
        assert!(
            matches!(
                DeepLink::parse(&input),
                Err(DeepLinkError::InvalidEncoding(_))
            ),
            "parser accepted {input}"
        );
    }
    assert!(DeepLink::parse("app://docs?file=report%20final.docx").is_ok());
}

#[test]
fn capability_table_matches_the_parser() {
    let table = super::surface_capabilities();
    assert_eq!(table.len(), 3);
    for capability in table {
        assert_eq!(capability.app_id, capability.surface.app_id());
        assert_eq!(capability.accepted_params, capability.surface.accepted_params());
        for param in OfficeParam::all() {
            assert_eq!(
                capability.accepted_params.contains(param),
                capability.surface.accepts(*param)
            );
        }
    }
    let slides = table
        .iter()
        .find(|capability| capability.surface == OfficeSurface::Slides)
        .expect("slides capability");
    assert!(!slides.accepted_params.contains(&OfficeParam::VmPath));

    let description = super::describe_capabilities();
    assert!(description.contains("app://docs"));
    assert!(description.contains("app://sheet"));
    assert!(description.contains("app://slides"));
    assert!(description.contains("vmConvId"));
    assert_eq!(description, super::describe_capabilities());
}
