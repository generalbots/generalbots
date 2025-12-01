# Bot Authentication

This section describes how General Bots handles bot authentication through its session-based architecture. Unlike traditional systems where bots might have independent credentials, General Bots implements a model where bots operate within the context of authenticated user sessions.

## Overview

Bot authentication in General Bots follows a fundamentally different approach from conventional bot platforms. Rather than assigning credentials directly to bots, the system ties all bot operations to user sessions. When a user authenticates through the Directory Service, they gain access to interact with bots based on their organizational membership and assigned permissions. This design eliminates the complexity of managing separate bot credentials while maintaining robust security through user-based access control.

The key principle underlying this architecture is that bots are resources accessed by users, not independent actors with their own identities. This approach simplifies security management and creates a clear audit trail linking all bot activities to specific authenticated users.

## Bot Registration

When the system bootstraps, bots are registered in the database through an automated discovery process. The system scans the `templates/` directory for any folder ending in `.gbai` and creates corresponding entries in the database.

### Database Storage

Each bot entry in the `bots` table contains a UUID primary key that uniquely identifies the bot, the bot's display name, an organization association that determines which users can access it, and timestamps tracking when the bot was created and last modified. This minimal schema reflects the philosophy that bots themselves don't require authentication credentials—they simply need to be identifiable and associable with organizations.

### Configuration Management

Bot-specific settings are stored separately in the `bot_configuration` table, which maintains key-value pairs loaded from the bot's `config.csv` file. This table holds runtime parameters, feature flags, LLM configuration, and any other settings that control the bot's behavior. By separating configuration from the core bot record, administrators can update settings without affecting the bot's fundamental identity or registration status.

## Session-Based Bot Access

The session-based access model forms the foundation of how users interact with bots. When a user wants to communicate with a bot, they must first authenticate through the Directory Service using standard OAuth2/OIDC flows. Once authenticated, the user can select from available bots based on their permissions, and the system creates a session that links that specific user to their chosen bot.

### Session Structure

The `user_sessions` table maintains the critical relationship between users and bots. Each session record contains a unique identifier, references to both the user and the selected bot, a session token for subsequent requests, and an expiration timestamp. All operations within that session are automatically scoped to the associated bot, preventing any accidental or intentional cross-bot data access.

This session structure means that when a user sends a message or requests information, the system automatically knows which bot should handle the request and which data stores should be queried. The session token serves as proof of both user authentication and bot selection, streamlining the authorization process for each subsequent request.

## Data Isolation

General Bots implements strict data isolation between bots to ensure that information from one bot cannot leak to another. Each bot maintains its own isolated storage for message history, memories, knowledge bases, configuration, and drive bucket contents.

### Cross-Bot Protection

The isolation model works at multiple levels. Sessions are locked to a single bot for their entire duration, meaning the system cannot accidentally route requests to the wrong bot. All database queries include the bot identifier as a filter condition, ensuring that even if a bug existed in the application logic, the database layer would prevent cross-bot data access. Storage buckets in the drive system are segregated by bot, with each bot's files residing in a dedicated bucket that other bots cannot access.

This defense-in-depth approach means that data isolation doesn't depend on any single mechanism being perfect. Multiple independent safeguards work together to maintain separation between bots.

## Bot Discovery and Selection

Users access bots through a discovery process that respects organizational boundaries and permission assignments. The available bots for any given user depend on their organization membership, any direct bot assignments they've received, whether specific bots are marked as publicly available, and their role-based access permissions.

When starting a new conversation, users are presented with a list of bots they're authorized to access. After selecting a bot, the system creates a new session linking the user to that bot, loads the bot's context including its configuration and any persistent memories, and the conversation begins with the bot's welcome message or startup script.

## Bot Lifecycle

Understanding the bot lifecycle helps administrators manage their bot deployments effectively. Bots move through several states from creation to active operation.

### Creation Process

During the bootstrap process, the system discovers bot templates and registers them in the database. For each template found, the system creates a bot record with generated identifiers, loads configuration from the bot's `config.csv` file, uploads the bot's resources to the drive storage system, and indexes any knowledge base documents into the vector database. This automated process means that deploying a new bot is as simple as adding its folder to the templates directory and restarting the server.

### Activation Requirements

A bot becomes active and available for user access when its registration is complete, its configuration passes validation, all required resources are available in storage, and no critical errors occurred during initialization. If any of these conditions aren't met, the bot remains in an inactive state and won't appear in users' available bot lists.

### Updating Bots

Bot updates follow a similar automated process. Changes to configuration files are detected and applied, modified scripts are reloaded, and knowledge base updates trigger reindexing. Importantly, none of these updates require any authentication changes because bots don't have their own credentials to manage.

## Permission Levels

Bot access is controlled through a hierarchy of visibility settings that administrators configure per bot. At the most open level, public bots can be accessed by anyone with a valid user account. Organization-level bots restrict access to members of the bot's associated organization. Private bots limit access to specifically assigned users. Admin-level bots require administrative privileges to access.

These permission levels work in conjunction with the Directory Service's group and role system, allowing fine-grained control over who can access which bots within an organization.

## Configuration Settings

Bot identity and access configuration are specified in the bot's `config.csv` file. The identity settings include the bot's display name and its organization association. Access configuration specifies the visibility level, which roles are permitted to access the bot, and operational limits like maximum concurrent sessions.

For example, a customer service bot might be configured with organization-level access, allowing any authenticated member of the organization to interact with it, while an HR bot might restrict access to members of the HR role group.

## Security Considerations

The design decision to not give bots their own credentials has significant security implications, all of them positive. Bots cannot authenticate independently, which means there's no possibility of a bot's credentials being compromised or misused. Every bot operation requires a valid user context, creating a complete audit trail. There's no mechanism for unauthorized bot-to-bot communication because bots can't initiate actions without a user session.

### Preventing Bot Impersonation

Because bots have no credentials, they cannot be impersonated through stolen credentials. An attacker would need to compromise an actual user account to interact with a bot, and even then, their actions would be logged against that user account. This makes detecting and investigating security incidents straightforward—every bot interaction traces back to a specific authenticated user.

## API Integration

All programmatic access to bots follows the same user-authenticated model as interactive access. API requests must include a valid user session token in the Authorization header, along with the target bot identifier in the request body or URL.

There are no separate bot API keys or service accounts for bot access. This uniformity simplifies the security model and ensures that API access receives the same level of auditing and access control as interactive access through the web interface.

## Multi-Bot Scenarios

Users who need to work with multiple bots can do so through several mechanisms. They can end their current bot session and start a new one with a different bot, with their conversation context switching to the new bot while history from each bot remains preserved separately. For users who need simultaneous access to multiple bots, the system supports concurrent sessions with different session identifiers, separate conversation contexts, and fully isolated data access.

This flexibility allows power users to leverage multiple bots for different tasks without the complexity of managing separate credentials or authentication contexts.

## Monitoring and Auditing

Administrators can monitor bot access patterns through built-in metrics and logging capabilities. Authentication metrics track sessions per bot, user engagement levels, access attempts, and permission denials. Audit logging captures session creation events, bot selection actions, configuration changes, and any access violations.

These monitoring capabilities support both operational oversight and compliance requirements, providing the visibility needed to understand how bots are being used across the organization.

## Best Practices

Successful bot deployment follows several established patterns. Organizing bots by organization groups them logically and simplifies permission management. Configuring appropriate access levels ensures that sensitive bots aren't accidentally exposed to unauthorized users. Monitoring usage patterns helps identify both popular bots that might need additional resources and underutilized bots that might need better documentation or training. Regular permission audits ensure that access levels remain appropriate as organizational roles change. Maintaining documentation for each bot helps users understand what each bot can do and when to use it. Testing data isolation periodically verifies that the security boundaries between bots remain intact.

## Troubleshooting Common Issues

When users report that a bot isn't accessible, several common causes should be investigated. The user might not be a member of the bot's organization, they might lack sufficient permissions for the bot's access level, the bot might not have completed its activation process, or there might be a configuration error preventing the bot from loading properly.

Session-related issues typically stem from expired sessions requiring re-authentication, invalid bot identifiers in API requests, concurrent session limits being exceeded, or database connectivity problems preventing session validation.

## Implementation Notes

Bot authentication is not implemented as a separate module but is integrated throughout the session management, user authentication, and database query systems. This integration reflects the fundamental design principle that bot access is a function of user authentication rather than an independent system.

Future versions might consider enhancements such as bot-specific API tokens for automated workflows, service accounts for scheduled bot operations, controlled bot-to-bot communication for complex scenarios, and webhook authentication for external system integration. However, any such features would be implemented as extensions of the user-session model rather than as independent bot credentials.

## Summary

The bot authentication model in General Bots achieves security through simplicity. By tying all bot access to authenticated user sessions, the system eliminates an entire class of credential management problems while maintaining complete auditability of all bot interactions. This design allows organizations to focus on building useful bots rather than managing complex authentication infrastructure, while still meeting enterprise security requirements.