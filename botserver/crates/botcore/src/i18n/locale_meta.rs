//! Per-locale presentation metadata (#2087, epic #2096).
//!
//! A language picker needs more than a BCP-47 code: the name in its own script
//! (a user who cannot read English must still find "日本語"), the text
//! direction, and the CLDR date/number patterns so a client can format values
//! without hard-coding `MM/DD/YYYY` or `1,234.56`.
//!
//! The catalogue (`botlib/locales/*`) is the source of truth for *which*
//! locales exist; this module only decorates them. A locale directory added
//! without an entry here still appears in the API, with a derived name and the
//! English formats, so a new language is never hidden by a missing table row.

use serde::Serialize;

/// Presentation metadata for one shipped locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocaleMetadata {
    /// BCP-47 code, identical to the catalogue directory name.
    pub code: String,
    /// The language's own name, so the picker is readable in every locale.
    pub native_name: String,
    /// `ltr` or `rtl` — the shell sets `dir` from this.
    pub dir: String,
    /// CLDR date pattern (e.g. `dd/MM/yyyy`) for `Intl.DateTimeFormat`.
    pub date_format: String,
    /// CLDR decimal pattern (e.g. `#.##0,###`) for `Intl.NumberFormat`.
    pub number_format: String,
}

/// The eight locales the catalogue ships, in the order the picker shows them.
const SHIPPED: &[(&str, &str, &str, &str)] = &[
    // (code, native_name, date_format, number_format)
    ("en", "English", "MM/dd/yyyy", "#,##0.###"),
    ("pt-BR", "Português (Brasil)", "dd/MM/yyyy", "#.##0,###"),
    ("es", "Español", "dd/MM/yyyy", "#.##0,###"),
    ("fr", "Français", "dd/MM/yyyy", "# ##0,###"),
    ("de", "Deutsch", "dd.MM.yyyy", "#.##0,###"),
    ("ja", "日本語", "yyyy/MM/dd", "#,##0.###"),
    ("ko", "한국어", "yyyy. MM. dd.", "#,##0.###"),
    ("zh-CN", "简体中文", "yyyy/M/d", "#,##0.###"),
];

/// Languages written right-to-left. None of the shipped locales is RTL today;
/// the set exists so adding `ar`, `he`, `fa` or `ur` needs no code change.
const RTL_LANGUAGES: &[&str] = &[
    "ar", "arc", "ckb", "dv", "fa", "he", "ks", "ps", "sd", "ug", "ur", "yi",
];

/// English formats are the fallback for a locale without an entry above.
const DEFAULT_DATE_FORMAT: &str = "MM/dd/yyyy";
const DEFAULT_NUMBER_FORMAT: &str = "#,##0.###";

/// `true` when the language subtag is written right-to-left.
#[must_use]
pub fn is_rtl(code: &str) -> bool {
    let language = code
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    RTL_LANGUAGES.contains(&language.as_str())
}

/// Metadata for one locale code, with a derived fallback.
///
/// The fallback keeps the contract total: every code the catalogue reports gets
/// an object with all five fields, so a client never has to guard against a
/// missing entry.
#[must_use]
pub fn metadata_for(code: &str) -> LocaleMetadata {
    let trimmed = code.trim();
    let shipped = SHIPPED
        .iter()
        .find(|(shipped_code, ..)| shipped_code.eq_ignore_ascii_case(trimmed));

    let (canonical, native_name, date_format, number_format) = match shipped {
        Some((shipped_code, native_name, date_format, number_format)) => {
            (*shipped_code, *native_name, *date_format, *number_format)
        }
        // An unknown code is shown as written rather than hidden: a locale
        // directory that landed without metadata still reaches the picker.
        None => (
            trimmed,
            trimmed,
            DEFAULT_DATE_FORMAT,
            DEFAULT_NUMBER_FORMAT,
        ),
    };

    LocaleMetadata {
        code: canonical.to_string(),
        native_name: native_name.to_string(),
        dir: if is_rtl(trimmed) { "rtl" } else { "ltr" }.to_string(),
        date_format: date_format.to_string(),
        number_format: number_format.to_string(),
    }
}

/// Metadata for every locale the catalogue loaded, preserving its order.
#[must_use]
pub fn metadata_all(codes: &[String]) -> Vec<LocaleMetadata> {
    codes.iter().map(|code| metadata_for(code)).collect()
}

#[cfg(test)]
mod tests {
    use super::{is_rtl, metadata_all, metadata_for};

    const SHIPPED_CODES: [&str; 8] = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

    #[test]
    fn every_shipped_locale_has_native_name_and_formats() {
        for code in SHIPPED_CODES {
            let meta = metadata_for(code);
            assert_eq!(meta.code, code);
            assert!(!meta.native_name.is_empty(), "native name for {code}");
            assert_ne!(meta.native_name, code, "native name for {code} is the code");
            assert_eq!(meta.dir, "ltr", "direction for {code}");
            assert!(!meta.date_format.is_empty(), "date format for {code}");
            assert!(!meta.number_format.is_empty(), "number format for {code}");
        }
    }

    #[test]
    fn native_names_are_written_in_their_own_script() {
        // A picker that shows "Chinese" instead of 简体中文 is unusable for the
        // users it exists for, so the names are asserted literally.
        assert_eq!(metadata_for("pt-BR").native_name, "Português (Brasil)");
        assert_eq!(metadata_for("ja").native_name, "日本語");
        assert_eq!(metadata_for("ko").native_name, "한국어");
        assert_eq!(metadata_for("zh-CN").native_name, "简体中文");
        assert_eq!(metadata_for("de").native_name, "Deutsch");
    }

    #[test]
    fn date_and_number_formats_follow_the_locale() {
        assert_eq!(metadata_for("en").date_format, "MM/dd/yyyy");
        assert_eq!(metadata_for("pt-BR").date_format, "dd/MM/yyyy");
        assert_eq!(metadata_for("de").date_format, "dd.MM.yyyy");
        assert_eq!(metadata_for("ja").date_format, "yyyy/MM/dd");
        assert_eq!(metadata_for("en").number_format, "#,##0.###");
        assert_eq!(metadata_for("pt-BR").number_format, "#.##0,###");
    }

    #[test]
    fn unknown_locales_fall_back_without_hiding_them() {
        let meta = metadata_for("sv");
        assert_eq!(meta.code, "sv");
        assert_eq!(meta.native_name, "sv");
        assert_eq!(meta.dir, "ltr");
        assert_eq!(meta.date_format, "MM/dd/yyyy");
        assert_eq!(meta.number_format, "#,##0.###");

        // Empty input stays total instead of panicking on a slice.
        let empty = metadata_for("   ");
        assert_eq!(empty.code, "");
        assert_eq!(empty.dir, "ltr");
    }

    #[test]
    fn right_to_left_languages_are_detected_by_subtag() {
        for rtl in ["ar", "ar-EG", "he_IL", "fa", "ur", "UR-PK"] {
            assert!(is_rtl(rtl), "{rtl} is rtl");
            assert_eq!(metadata_for(rtl).dir, "rtl", "direction for {rtl}");
        }
        for ltr in ["en", "pt-BR", "zh-CN", "ja", ""] {
            assert!(!is_rtl(ltr), "{ltr} is ltr");
        }
    }

    #[test]
    fn metadata_all_preserves_the_catalogue_order() {
        let codes: Vec<String> = SHIPPED_CODES.iter().map(|c| (*c).to_string()).collect();
        let all = metadata_all(&codes);
        assert_eq!(all.len(), codes.len());
        let returned: Vec<&str> = all.iter().map(|meta| meta.code.as_str()).collect();
        assert_eq!(returned, codes);
        // The empty catalogue is not an error: the caller decides what to show.
        assert!(metadata_all(&[]).is_empty());
    }

    #[test]
    fn metadata_is_serialized_with_the_documented_field_names() {
        let value = serde_json::to_value(metadata_for("pt-BR")).expect("serializable");
        for field in ["code", "native_name", "dir", "date_format", "number_format"] {
            assert!(value.get(field).is_some(), "missing {field}");
        }
        assert_eq!(value["code"], "pt-BR");
        assert_eq!(value["dir"], "ltr");
    }
}
