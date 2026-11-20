//! Workbook-level locale and calculation settings (#2572).
//!
//! The local preview keeps these values in `gb-sheet-functions:<sheetId>`.
//! This module is the server-side counterpart: the wire shape is deliberately
//! identical to the preview state so a workbook JSON document can carry the
//! same object without a translation layer. It is a standalone type because
//! `Spreadsheet` is constructed exhaustively by the `botsheet` crate; adding a
//! field there is a separate, coordinated change.

use serde::{Deserialize, Serialize};

/// Formula locale. The string values match the preview's `LOCALES` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormulaLocale {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "es")]
    Es,
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "de")]
    De,
    #[serde(rename = "ja")]
    Ja,
    #[serde(rename = "ko")]
    Ko,
    #[serde(rename = "zh-CN")]
    ZhCn,
}

impl FormulaLocale {
    pub const ALL: [FormulaLocale; 8] = [
        FormulaLocale::En,
        FormulaLocale::PtBr,
        FormulaLocale::Es,
        FormulaLocale::Fr,
        FormulaLocale::De,
        FormulaLocale::Ja,
        FormulaLocale::Ko,
        FormulaLocale::ZhCn,
    ];

    /// Accepts the preview's exact spellings and common case/underscore forms.
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "pt-br" => FormulaLocale::PtBr,
            "es" => FormulaLocale::Es,
            "fr" => FormulaLocale::Fr,
            "de" => FormulaLocale::De,
            "ja" => FormulaLocale::Ja,
            "ko" => FormulaLocale::Ko,
            "zh-cn" => FormulaLocale::ZhCn,
            _ => FormulaLocale::En,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FormulaLocale::En => "en",
            FormulaLocale::PtBr => "pt-BR",
            FormulaLocale::Es => "es",
            FormulaLocale::Fr => "fr",
            FormulaLocale::De => "de",
            FormulaLocale::Ja => "ja",
            FormulaLocale::Ko => "ko",
            FormulaLocale::ZhCn => "zh-CN",
        }
    }

    /// Decimal mark used by the preview for this locale.
    pub fn decimal_separator(&self) -> char {
        if matches!(
            self,
            FormulaLocale::PtBr | FormulaLocale::Es | FormulaLocale::Fr | FormulaLocale::De
        ) {
            ','
        } else {
            '.'
        }
    }

    /// Thousands/grouping mark used by the preview for this locale.
    pub fn group_separator(&self) -> char {
        match self {
            FormulaLocale::PtBr | FormulaLocale::Es | FormulaLocale::De => '.',
            FormulaLocale::Fr => ' ',
            _ => ',',
        }
    }

    /// Excel convention: a comma decimal mark implies a semicolon argument
    /// separator. This is what a formula authored in the preview uses.
    pub fn argument_separator(&self) -> char {
        if self.decimal_separator() == ',' {
            ';'
        } else {
            ','
        }
    }

    pub fn separators(&self) -> Separators {
        Separators {
            decimal: self.decimal_separator(),
            group: self.group_separator(),
        }
    }
}

/// Decimal and grouping marks; serialized as the preview's `separators`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Separators {
    pub decimal: char,
    pub group: char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CalcMode {
    Auto,
    Manual,
}

impl CalcMode {
    pub fn parse(value: &str) -> Self {
        if value.eq_ignore_ascii_case("manual") {
            CalcMode::Manual
        } else {
            CalcMode::Auto
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CalcMode::Auto => "auto",
            CalcMode::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateSystem {
    #[serde(rename = "1900")]
    Excel1900,
    #[serde(rename = "1904")]
    Excel1904,
}

impl DateSystem {
    pub fn parse(value: &str) -> Self {
        if value == "1904" {
            DateSystem::Excel1904
        } else {
            DateSystem::Excel1900
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DateSystem::Excel1900 => "1900",
            DateSystem::Excel1904 => "1904",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IterativeSettings {
    pub enabled: bool,
    pub max_iterations: u32,
    pub tolerance: f64,
}

impl Default for IterativeSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_iterations: 100,
            tolerance: 1e-7,
        }
    }
}

/// Settings persisted per workbook. Field names mirror the preview state so
/// the same JSON object can move from localStorage to the server unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkbookSettings {
    pub version: u32,
    /// Kept for wire compatibility. Server-persisted workbooks set it false.
    pub local_only: bool,
    pub mode: CalcMode,
    pub iterative: IterativeSettings,
    pub precision_as_displayed: bool,
    pub date_system: DateSystem,
    pub locale: FormulaLocale,
    pub separators: Separators,
}

impl Default for WorkbookSettings {
    fn default() -> Self {
        Self {
            version: 1,
            local_only: false,
            mode: CalcMode::Auto,
            iterative: IterativeSettings::default(),
            precision_as_displayed: false,
            date_system: DateSystem::Excel1900,
            locale: FormulaLocale::En,
            separators: FormulaLocale::En.separators(),
        }
    }
}

impl WorkbookSettings {
    /// Applies the preview's bounds: 1..=1000 iterations, positive tolerance
    /// and two distinct separator marks.
    pub fn normalized(mut self) -> Self {
        self.iterative.max_iterations = self.iterative.max_iterations.clamp(1, 1000);
        if !self.iterative.tolerance.is_finite() || self.iterative.tolerance <= 0.0 {
            self.iterative.tolerance = 1e-7;
        }
        if self.separators.decimal == self.separators.group
            || !self.separators.decimal.is_ascii_punctuation()
            || !self.separators.group.is_ascii_punctuation()
        {
            self.separators = self.locale.separators();
        }
        self
    }

    /// Enables iterative recalculation with the preview's test settings.
    pub fn iterative_enabled() -> Self {
        Self {
            iterative: IterativeSettings {
                enabled: true,
                max_iterations: 100,
                tolerance: 1e-9,
            },
            ..Self::default()
        }
    }
}

/// Reads `settings` from a persisted workbook JSON document. Documents saved
/// before #2572 have no such key and silently get the defaults.
pub fn settings_from_workbook_json(json: &str) -> WorkbookSettings {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return WorkbookSettings::default();
    };
    settings_from_value(value)
}

/// Byte-oriented counterpart used by the Drive load path, where the document
/// arrives as raw object storage bytes.
pub fn settings_from_workbook_bytes(bytes: &[u8]) -> WorkbookSettings {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return WorkbookSettings::default();
    };
    settings_from_value(value)
}

fn settings_from_value(value: serde_json::Value) -> WorkbookSettings {
    let Some(object) = value.as_object() else {
        return WorkbookSettings::default();
    };
    object
        .get("settings")
        .and_then(|raw| serde_json::from_value::<WorkbookSettings>(raw.clone()).ok())
        .unwrap_or_default()
        .normalized()
}

/// Writes `settings` into a persisted workbook JSON document without changing
/// the shape of `Spreadsheet` (whose literals are owned by the `botsheet`
/// crate). The save hook can call this before putting the object in Drive.
pub fn embed_settings_in_workbook_json(
    json: &str,
    settings: &WorkbookSettings,
) -> Result<String, String> {
    let mut value: serde_json::Value =
        serde_json::from_str(json).map_err(|error| format!("invalid workbook JSON: {error}"))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "workbook JSON must be an object".to_string())?;
    let encoded = serde_json::to_value(settings)
        .map_err(|error| format!("settings serialization failed: {error}"))?;
    object.insert("settings".to_string(), encoded);
    serde_json::to_string(&value).map_err(|error| format!("workbook serialization failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_shape_matches_the_preview() {
        let json = serde_json::to_string(&WorkbookSettings::default()).unwrap_or_default();
        for key in [
            "version",
            "localOnly",
            "mode",
            "iterative",
            "precisionAsDisplayed",
            "dateSystem",
            "locale",
            "separators",
        ] {
            assert!(json.contains(&format!("\"{key}\"")), "missing {key}: {json}");
        }
        assert!(json.contains("\"maxIterations\""));
        assert!(json.contains("\"tolerance\""));
        assert!(json.contains("\"decimal\""));
        assert!(json.contains("\"group\""));
    }

    #[test]
    fn all_locales_round_trip_with_their_separators() {
        for locale in FormulaLocale::ALL {
            let settings = WorkbookSettings {
                locale,
                separators: locale.separators(),
                ..WorkbookSettings::default()
            };
            let json = serde_json::to_string(&settings).unwrap_or_default();
            let back: WorkbookSettings = serde_json::from_str(&json).unwrap_or_default();
            assert_eq!(back.locale, locale);
            assert_eq!(back.separators, locale.separators());
            assert_eq!(back.locale.as_str(), locale.as_str());
        }
        assert_eq!(FormulaLocale::parse("PT_PT"), FormulaLocale::En);
        assert_eq!(FormulaLocale::parse("pt-BR"), FormulaLocale::PtBr);
        assert_eq!(FormulaLocale::PtBr.argument_separator(), ';');
        assert_eq!(FormulaLocale::En.argument_separator(), ',');
    }

    #[test]
    fn settings_survive_workbook_json_round_trip() {
        let settings = WorkbookSettings {
            mode: CalcMode::Manual,
            precision_as_displayed: true,
            date_system: DateSystem::Excel1904,
            locale: FormulaLocale::PtBr,
            separators: FormulaLocale::PtBr.separators(),
            iterative: IterativeSettings {
                enabled: true,
                max_iterations: 250,
                tolerance: 1e-9,
            },
            ..WorkbookSettings::default()
        };
        let embedded =
            embed_settings_in_workbook_json(r#"{"id":"s1","name":"Book"}"#, &settings)
                .unwrap_or_default();
        let restored = settings_from_workbook_json(&embedded);
        assert_eq!(restored, settings);
        assert_eq!(restored.date_system.as_str(), "1904");
        assert_eq!(restored.mode.as_str(), "manual");
    }

    #[test]
    fn legacy_workbook_json_gets_defaults() {
        let settings = settings_from_workbook_json(r#"{"id":"old"}"#);
        assert_eq!(settings, WorkbookSettings::default());
        assert_eq!(settings_from_workbook_json("not json"), WorkbookSettings::default());
    }

    #[test]
    fn normalize_clamps_iterative_bounds() {
        let settings = WorkbookSettings {
            iterative: IterativeSettings {
                enabled: true,
                max_iterations: 5000,
                tolerance: -1.0,
            },
            separators: Separators {
                decimal: ',',
                group: ',',
            },
            locale: FormulaLocale::Fr,
            ..WorkbookSettings::default()
        }
        .normalized();
        assert_eq!(settings.iterative.max_iterations, 1000);
        assert_eq!(settings.iterative.tolerance, 1e-7);
        assert_eq!(settings.separators, FormulaLocale::Fr.separators());
    }
}
