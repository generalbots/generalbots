//! Localized reason phrases interpolated into the upload rejection messages.
//!
//! The catalogue templates (`error-file-archive-unsafe`,
//! `error-file-upload-failed`) take the reason as an argument; these phrases
//! follow the request language so the whole message reads in one language.
//! Portuguese gets its own text, every other language keeps the English one.

use botlib::i18n::Locale;

/// A reason the upload gate can give.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reason {
    EncryptedEntry,
    NestedArchive,
    MacroPart,
    CompressionRatio,
    UnsafeReference,
    ScannerMissing,
    ScanFailed,
    MalwareDetected,
}

/// The phrase for `reason` in the request language.
pub(crate) fn phrase(locale: &Locale, reason: Reason) -> &'static str {
    if locale.language().eq_ignore_ascii_case("pt") {
        portuguese(reason)
    } else {
        english(reason)
    }
}

fn english(reason: Reason) -> &'static str {
    match reason {
        Reason::EncryptedEntry => "encrypted entry",
        Reason::NestedArchive => "nested archive",
        Reason::MacroPart => "macro-bearing part",
        Reason::CompressionRatio => "compression ratio too high",
        Reason::UnsafeReference => "unsafe external reference",
        Reason::ScannerMissing => "no malware scanner is configured",
        Reason::ScanFailed => "the malware scan could not be completed",
        Reason::MalwareDetected => "malware detected",
    }
}

fn portuguese(reason: Reason) -> &'static str {
    match reason {
        Reason::EncryptedEntry => "contém um item criptografado",
        Reason::NestedArchive => "contém outro arquivo compactado dentro dele",
        Reason::MacroPart => "contém macros",
        Reason::CompressionRatio => "taxa de compressão alta demais",
        Reason::UnsafeReference => "referência externa insegura",
        Reason::ScannerMissing => "nenhum verificador de malware está configurado",
        Reason::ScanFailed => "não foi possível concluir a verificação de malware",
        Reason::MalwareDetected => "foi encontrado malware no arquivo",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(tag: &str) -> Locale {
        Locale::new(tag).unwrap_or_default()
    }

    #[test]
    fn portuguese_requests_get_portuguese_phrases() {
        assert_eq!(
            phrase(&locale("pt-BR"), Reason::ScanFailed),
            "não foi possível concluir a verificação de malware"
        );
    }

    #[test]
    fn other_requests_keep_the_english_phrases() {
        assert_eq!(
            phrase(&locale("en"), Reason::ScanFailed),
            "the malware scan could not be completed"
        );
        assert_eq!(phrase(&locale("de"), Reason::MalwareDetected), "malware detected");
    }

    #[test]
    fn every_reason_has_distinct_text_in_both_languages() {
        let all = [
            Reason::EncryptedEntry,
            Reason::NestedArchive,
            Reason::MacroPart,
            Reason::CompressionRatio,
            Reason::UnsafeReference,
            Reason::ScannerMissing,
            Reason::ScanFailed,
            Reason::MalwareDetected,
        ];
        for reason in all {
            assert_ne!(english(reason), portuguese(reason), "{reason:?}");
        }
    }
}
