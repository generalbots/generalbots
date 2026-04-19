# Core Package - Core System Functionality

## Purpose
Contains core system functionality and infrastructure. Provides the foundation for all other packages and handles essential operations.

## Key Files
- **bot_database.rs**: Bot database management
- **config_reload.rs**: Configuration reload functionality
- **features.rs**: Feature flag management
- **i18n.rs**: Internationalization (i18n) support
- **large_org_optimizer.rs**: Performance optimization for large organizations
- **manifest.rs**: Application manifest management
- **middleware.rs**: Custom middleware
- **mod.rs**: Module entry point and exports
- **organization.rs**: Organization management
- **organization_invitations.rs**: Invitation system
- **organization_rbac.rs**: RBAC for organizations
- **performance.rs**: Performance monitoring
- **product.rs**: Product information management
- **rate_limit.rs**: Rate limiting
- **urls.rs**: URL utilities

## Submodules
- **automation/**: Automation framework
- **bootstrap/**: System bootstrap process
- **bot/**: Bot management
- **config/**: Configuration management
- **directory/**: Directory services
- **dns/**: DNS integration
- **incus/**: Incus container management
- **kb/**: Knowledge base
- **oauth/**: OAuth2 integration
- **package_manager/**: Package management
- **secrets/**: Secrets management
- **session/**: Session management
- **shared/**: Shared utilities

## Core Features

### Configuration Management
```rust
use crate::core::config::Config;

// Load configuration
let config = Config::load().expect("Failed to load configuration");

// Get specific setting
let port = config.server.port;
```

### Organization Management
```rust
use crate::core::organization::OrganizationService;

let org_service = OrganizationService::new();

// Create organization
let org = org_service.create_organization(
    "Acme Corporation".to_string(),
    "acme".to_string()
).await?;

// Get organization
let org = org_service.get_organization(org_id).await?;
```

### Performance Monitoring
```rust
use crate::core::performance::PerformanceMonitor;

let monitor = PerformanceMonitor::new();

// Track operation
let result = monitor.track("database_query", || {
    // Database query operation
    execute_query()
}).await;

// Get performance metrics
let metrics = monitor.get_metrics().await;
```

## Architecture
The core package is designed with:
- **Layered architecture**: Separation of concerns
- **Dependency injection**: Testability and flexibility
- **Error handling**: Comprehensive error types
- **Configuration**: Environment-based configuration

## System Bootstrap
The bootstrap process is defined in `bootstrap/` module:
1. Loads configuration
2. Initializes database connections
3. Sets up services
4. Starts the server
5. Initializes system components

## Performance Optimization
- Large organization optimization
- Connection pooling
- Caching strategies
- Asynchronous operations

## Error Handling
Core errors are defined in `crate::error` module:
- `CoreError`: General core errors
- `ConfigError`: Configuration errors
- `DatabaseError`: Database errors
- `OrganizationError`: Organization errors

## Testing
Core functionality is tested with:
- Unit tests for each module
- Integration tests for system flows
- Performance benchmarks
- Error handling tests