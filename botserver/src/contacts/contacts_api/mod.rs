mod error;
mod migration;
mod service;
mod types;
mod handlers;

pub use error::*;
pub use migration::*;
pub use service::*;
pub use types::*;
pub use handlers::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_status_display() {
        assert_eq!(ContactStatus::Active.to_string(), "active");
        assert_eq!(ContactStatus::Lead.to_string(), "lead");
        assert_eq!(ContactStatus::Customer.to_string(), "customer");
    }

    #[test]
    fn test_contact_source_display() {
        assert_eq!(ContactSource::Manual.to_string(), "manual");
        assert_eq!(ContactSource::Import.to_string(), "import");
        assert_eq!(ContactSource::WebForm.to_string(), "web_form");
    }

    #[test]
    fn test_activity_type_display() {
        assert_eq!(ActivityType::Email.to_string(), "email");
        assert_eq!(ActivityType::Meeting.to_string(), "meeting");
        assert_eq!(ActivityType::Created.to_string(), "created");
    }

    #[test]
    fn test_contacts_error_display() {
        assert_eq!(ContactsError::NotFound.to_string(), "Contact not found");
        assert_eq!(ContactsError::CreateFailed.to_string(), "Failed to create contact");
    }

    #[test]
    fn test_contact_status_default() {
        let status = ContactStatus::default();
        assert_eq!(status, ContactStatus::Active);
    }

    #[test]
    fn test_import_error_creation() {
        let err = ImportError {
            line: 5,
            field: Some("email".to_string()),
            message: "Invalid email format".to_string(),
        };
        assert_eq!(err.line, 5);
        assert_eq!(err.field, Some("email".to_string()));
    }

    #[test]
    fn test_export_result_creation() {
        let result = ExportResult {
            success: true,
            data: "test data".to_string(),
            content_type: "text/csv".to_string(),
            filename: "contacts.csv".to_string(),
            contact_count: 10,
        };
        assert!(result.success);
        assert_eq!(result.contact_count, 10);
    }
}
