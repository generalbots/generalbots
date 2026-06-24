use crate::connector_types::*;

pub fn get_all_templates() -> Vec<ConnectorTemplate> {
    vec![
        salesforce_template(),
        sap_template(),
        totvs_template(),
        bling_template(),
        shopify_template(),
        woo_commerce_template(),
        mysql_template(),
        postgres_template(),
        rest_api_template(),
        google_sheets_template(),
        sharepoint_template(),
    ]
}

pub fn get_template(connector_type: &str) -> Option<ConnectorTemplate> {
    get_all_templates().into_iter().find(|t| t.connector_type.to_string() == connector_type)
}

fn salesforce_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "salesforce".into(),
        name: "Salesforce".into(),
        description: "CRM platform for sales, service, marketing, and analytics".into(),
        connector_type: ConnectorType::Salesforce,
        icon: "cloud".into(),
        auth_type: AuthType::OAuth2,
        auth_help: "Configure OAuth2 in Salesforce Connected App. Use https://login.salesforce.com as base URL.".into(),
        default_endpoints: vec![
            endpoint("Contacts", "GET", "/services/data/v52.0/query?q=SELECT+Id,FirstName,LastName,Email,Phone+FROM+Contact"),
            endpoint("Leads", "GET", "/services/data/v52.0/query?q=SELECT+Id,FirstName,LastName,Email,Phone,Company+FROM+Lead"),
            endpoint("Opportunities", "GET", "/services/data/v52.0/query?q=SELECT+Id,Name,Amount,StageName,CloseDate+FROM+Opportunity"),
            endpoint("Accounts", "GET", "/services/data/v52.0/query?q=SELECT+Id,Name,Type,Industry,Phone+FROM+Account"),
        ],
        default_schedule: Some("0 */6 * * *".into()),
        color: "#00A1E0".into(),
    }
}

fn sap_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "sap".into(),
        name: "SAP Business One".into(),
        description: "Enterprise ERP for business management".into(),
        connector_type: ConnectorType::Sap,
        icon: "database".into(),
        auth_type: AuthType::Basic,
        auth_help: "Use SAP Service Layer credentials. Base URL format: https://host:port/b1s/v1/".into(),
        default_endpoints: vec![
            endpoint("BusinessPartners", "GET", "/b1s/v1/BusinessPartners"),
            endpoint("Items", "GET", "/b1s/v1/Items"),
            endpoint("Orders", "GET", "/b1s/v1/Orders"),
            endpoint("Invoices", "GET", "/b1s/v1/Invoices"),
        ],
        default_schedule: Some("0 */12 * * *".into()),
        color: "#003366".into(),
    }
}

fn totvs_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "totvs".into(),
        name: "TOTVS Protheus".into(),
        description: "Brazilian ERP by TOTVS".into(),
        connector_type: ConnectorType::Totvs,
        icon: "briefcase".into(),
        auth_type: AuthType::ApiKey,
        auth_help: "Configure API Key in TOTVS REST bridge. Base URL: http://host:port/rest/".into(),
        default_endpoints: vec![
            endpoint("Clients SA1", "GET", "/rest/api/sa1/"),
            endpoint("Products SB1", "GET", "/rest/api/sb1/"),
            endpoint("Sales Orders SC7", "GET", "/rest/api/sc7/"),
        ],
        default_schedule: Some("0 */12 * * *".into()),
        color: "#ED1C24".into(),
    }
}

fn bling_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "bling".into(),
        name: "Bling".into(),
        description: "Brazilian business management ERP".into(),
        connector_type: ConnectorType::Bling,
        icon: "shopping-bag".into(),
        auth_type: AuthType::OAuth2,
        auth_help: "Create app in Bling to get Client ID and Secret. Authorization code flow.".into(),
        default_endpoints: vec![
            endpoint("Contacts", "GET", "/api/contatos"),
            endpoint("Products", "GET", "/api/produtos"),
            endpoint("Orders", "GET", "/api/pedidos/vendas"),
            endpoint("TaxCategories", "GET", "/api/categorias/produtos"),
        ],
        default_schedule: Some("0 */6 * * *".into()),
        color: "#3F51B5".into(),
    }
}

fn shopify_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "shopify".into(),
        name: "Shopify".into(),
        description: "E-commerce platform for online stores".into(),
        connector_type: ConnectorType::Shopify,
        icon: "shopping-cart".into(),
        auth_type: AuthType::ApiKey,
        auth_help: "Admin API key from Shopify Admin. Base URL: https://{store}.myshopify.com/admin/api/2024-01/".into(),
        default_endpoints: vec![
            endpoint("Customers", "GET", "/admin/api/2024-01/customers.json"),
            endpoint("Products", "GET", "/admin/api/2024-01/products.json"),
            endpoint("Orders", "GET", "/admin/api/2024-01/orders.json"),
            endpoint("Inventory", "GET", "/admin/api/2024-01/inventory_items.json"),
        ],
        default_schedule: Some("0 */4 * * *".into()),
        color: "#96BF48".into(),
    }
}

fn woo_commerce_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "woocommerce".into(),
        name: "WooCommerce".into(),
        description: "Open-source e-commerce for WordPress".into(),
        connector_type: ConnectorType::WooCommerce,
        icon: "shopping-cart".into(),
        auth_type: AuthType::Basic,
        auth_help: "Consumer Key and Consumer Secret from WooCommerce > Settings > Advanced > REST API.".into(),
        default_endpoints: vec![
            endpoint("Customers", "GET", "/wp-json/wc/v3/customers"),
            endpoint("Products", "GET", "/wp-json/wc/v3/products"),
            endpoint("Orders", "GET", "/wp-json/wc/v3/orders"),
            endpoint("Categories", "GET", "/wp-json/wc/v3/products/categories"),
        ],
        default_schedule: Some("0 */6 * * *".into()),
        color: "#7F54B3".into(),
    }
}

fn mysql_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "mysql".into(),
        name: "MySQL Database".into(),
        description: "Relational database by Oracle".into(),
        connector_type: ConnectorType::MySql,
        icon: "database".into(),
        auth_type: AuthType::Basic,
        auth_help: "Connection string: mysql://user:pass@host:3306/dbname".into(),
        default_endpoints: Vec::new(),
        default_schedule: Some("0 */12 * * *".into()),
        color: "#00758F".into(),
    }
}

fn postgres_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "postgres".into(),
        name: "PostgreSQL Database".into(),
        description: "Advanced open-source relational database".into(),
        connector_type: ConnectorType::Postgres,
        icon: "database".into(),
        auth_type: AuthType::Basic,
        auth_help: "Connection string: postgresql://user:pass@host:5432/dbname".into(),
        default_endpoints: Vec::new(),
        default_schedule: Some("0 */12 * * *".into()),
        color: "#336791".into(),
    }
}

fn rest_api_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "rest_api".into(),
        name: "REST API".into(),
        description: "Generic RESTful API connector".into(),
        connector_type: ConnectorType::RestApi,
        icon: "globe".into(),
        auth_type: AuthType::ApiKey,
        auth_help: "Configure base URL, authentication, and endpoint paths.".into(),
        default_endpoints: vec![
            endpoint("Default endpoint", "GET", "/api/v1/data"),
        ],
        default_schedule: None,
        color: "#10B981".into(),
    }
}

fn google_sheets_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "google_sheets".into(),
        name: "Google Sheets".into(),
        description: "Spreadsheet sheet by Google".into(),
        connector_type: ConnectorType::GoogleSheets,
        icon: "sheet".into(),
        auth_type: AuthType::OAuth2,
        auth_help: "OAuth2 client credentials from Google Cloud Console. Scopes: https://www.googleapis.com/auth/spreadsheets".into(),
        default_endpoints: vec![
            EndpointConfig {
                name: "Read rows".into(),
                method: "GET".into(),
                url: "/v4/spreadsheets/{sheet_id}/values/{range}".into(),
                headers: None,
                auth_type: AuthType::OAuth2,
                sync_direction: SyncDirection::Pull,
                field_mapping: Vec::new(),
                schedule: None,
                pagination: None,
            },
        ],
        default_schedule: Some("0 */12 * * *".into()),
        color: "#0F9D58".into(),
    }
}

fn sharepoint_template() -> ConnectorTemplate {
    ConnectorTemplate {
        id: "sharepoint".into(),
        name: "SharePoint Online".into(),
        description: "Microsoft 365 document management and collaboration platform".into(),
        connector_type: ConnectorType::SharePoint,
        icon: "cloud".into(),
        auth_type: AuthType::OAuth2,
        auth_help: "Azure AD app registration required. Scopes: Sites.ReadWrite.All, Files.ReadWrite.All".into(),
        default_endpoints: vec![
            EndpointConfig {
                name: "Site lists".into(),
                method: "GET".into(),
                url: "/v1.0/sites/{site_id}/lists".into(),
                headers: None,
                auth_type: AuthType::OAuth2,
                sync_direction: SyncDirection::Pull,
                field_mapping: Vec::new(),
                schedule: None,
                pagination: None,
            },
            EndpointConfig {
                name: "Documents".into(),
                method: "GET".into(),
                url: "/v1.0/sites/{site_id}/drive/root/children".into(),
                headers: None,
                auth_type: AuthType::OAuth2,
                sync_direction: SyncDirection::Pull,
                field_mapping: Vec::new(),
                schedule: None,
                pagination: None,
            },
        ],
        default_schedule: Some("0 */6 * * *".into()),
        color: "#0078D4".into(),
    }
}

fn endpoint(name: &str, method: &str, url: &str) -> EndpointConfig {
    EndpointConfig {
        name: name.to_string(),
        method: method.to_string(),
        url: url.to_string(),
        headers: None,
        auth_type: AuthType::ApiKey,
        sync_direction: SyncDirection::Pull,
        field_mapping: Vec::new(),
        schedule: None,
        pagination: None,
    }
}
