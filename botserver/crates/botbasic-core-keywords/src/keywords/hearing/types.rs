use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputType {
    Any,
    Email,
    Date,
    Name,
    Integer,
    Float,
    Boolean,
    Hour,
    Money,
    Mobile,
    Zipcode,
    Language,
    Cpf,
    Cnpj,
    QrCode,
    Login,
    Menu(Vec<String>),
    File,
    Image,
    Audio,
    Video,
    Document,
    Url,
    Uuid,
    Color,
    CreditCard,
    Password,
}

impl InputType {
    #[must_use]
    pub fn error_message(&self) -> String {
        match self {
            Self::Any => String::new(),
            Self::Email => {
                "Please enter a valid email address (e.g., user@example.com)".to_string()
            }
            Self::Date => "Please enter a valid date (e.g., 25/12/2024 or 2024-12-25)".to_string(),
            Self::Name => "Please enter a valid name (letters and spaces only)".to_string(),
            Self::Integer => "Please enter a valid whole number".to_string(),
            Self::Float => "Please enter a valid number".to_string(),
            Self::Boolean => "Please answer yes or no".to_string(),
            Self::Hour => "Please enter a valid time (e.g., 14:30 or 2:30 PM)".to_string(),
            Self::Money => "Please enter a valid amount (e.g., 100.00 or R$ 100,00)".to_string(),
            Self::Mobile => "Please enter a valid mobile number".to_string(),
            Self::Zipcode => "Please enter a valid ZIP/postal code".to_string(),
            Self::Language => "Please enter a valid language code (e.g., en, pt, es)".to_string(),
            Self::Cpf => "Please enter a valid CPF (11 digits)".to_string(),
            Self::Cnpj => "Please enter a valid CNPJ (14 digits)".to_string(),
            Self::QrCode => "Please send an image containing a QR code".to_string(),
            Self::Login => "Please complete the authentication process".to_string(),
            Self::Menu(options) => format!("Please select one of: {}", options.join(", ")),
            Self::File => "Please upload a file".to_string(),
            Self::Image => "Please send an image".to_string(),
            Self::Audio => "Please send an audio file or voice message".to_string(),
            Self::Video => "Please send a video".to_string(),
            Self::Document => "Please send a document (PDF, Word, etc.)".to_string(),
            Self::Url => "Please enter a valid URL".to_string(),
            Self::Uuid => "Please enter a valid UUID".to_string(),
            Self::Color => "Please enter a valid color (e.g., #FF0000 or red)".to_string(),
            Self::CreditCard => "Please enter a valid credit card number".to_string(),
            Self::Password => "Please enter a password (minimum 8 characters)".to_string(),
        }
    }

    #[must_use]
    pub fn parse_type(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "EMAIL" => Self::Email,
            "DATE" => Self::Date,
            "NAME" => Self::Name,
            "INTEGER" | "INT" | "NUMBER" => Self::Integer,
            "FLOAT" | "DECIMAL" | "DOUBLE" => Self::Float,
            "BOOLEAN" | "BOOL" => Self::Boolean,
            "HOUR" | "TIME" => Self::Hour,
            "MONEY" | "CURRENCY" | "AMOUNT" => Self::Money,
            "MOBILE" | "PHONE" | "TELEPHONE" => Self::Mobile,
            "ZIPCODE" | "ZIP" | "CEP" | "POSTALCODE" => Self::Zipcode,
            "LANGUAGE" | "LANG" => Self::Language,
            "CPF" => Self::Cpf,
            "CNPJ" => Self::Cnpj,
            "QRCODE" | "QR" => Self::QrCode,
            "LOGIN" | "AUTH" => Self::Login,
            "FILE" => Self::File,
            "IMAGE" | "PHOTO" | "PICTURE" => Self::Image,
            "AUDIO" | "VOICE" | "SOUND" => Self::Audio,
            "VIDEO" => Self::Video,
            "DOCUMENT" | "DOC" | "PDF" => Self::Document,
            "URL" | "LINK" => Self::Url,
            "UUID" | "GUID" => Self::Uuid,
            "COLOR" | "COLOUR" => Self::Color,
            "CREDITCARD" | "CARD" => Self::CreditCard,
            "PASSWORD" | "PASS" | "SECRET" => Self::Password,
            _ => Self::Any,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub normalized_value: String,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl ValidationResult {
    #[must_use]
    pub fn valid(value: String) -> Self {
        Self {
            is_valid: true,
            normalized_value: value,
            error_message: None,
            metadata: None,
        }
    }

    #[must_use]
    pub fn valid_with_metadata(value: String, metadata: serde_json::Value) -> Self {
        Self {
            is_valid: true,
            normalized_value: value,
            error_message: None,
            metadata: Some(metadata),
        }
    }

    #[must_use]
    pub fn invalid(error: String) -> Self {
        Self {
            is_valid: false,
            normalized_value: String::new(),
            error_message: Some(error),
            metadata: None,
        }
    }
}
