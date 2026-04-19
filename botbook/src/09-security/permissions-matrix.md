# Permissions Matrix

This chapter documents the permission system in General Bots, explaining how role-based access control governs what users can do within the platform. Understanding this permission model is essential for administrators configuring access policies and developers building applications that respect security boundaries.

## Understanding the Permission Model

General Bots implements a role-based access control (RBAC) system that integrates with Zitadel, the platform's Directory Service. The permission architecture consists of three interconnected layers that work together to determine what any given user can do.

At the highest level, realms establish permission boundaries that typically correspond to organizations. Within each realm, groups collect users who share common access needs. Permissions represent specific actions that can be granted to groups, and users inherit the combined permissions of all groups to which they belong.

This layered approach provides flexibility while maintaining manageability. Rather than assigning permissions directly to individual users, administrators create groups with appropriate permission sets and then add users to those groups. When a user's responsibilities change, their access can be adjusted simply by modifying their group memberships.

## User Context and System Context

APIs in General Bots operate in one of two security contexts, each with distinct characteristics and use cases.

User context operations execute on behalf of an authenticated user, using their OAuth token for authorization. When an API operates in user context, it sees and modifies only resources that belong to or are shared with that user. Reading files, sending messages, accessing calendars, managing tasks, and viewing email all occur in user context. The principle of least privilege applies naturally here—users can only access what they own or what has been explicitly shared with them.

System context operations execute on behalf of the bot or system itself, using a service account token. These operations have broader access because they serve cross-cutting concerns that don't belong to any individual user. Bot-initiated messages, scheduled task execution, system monitoring, cross-user analytics, and backup operations all require system context to function properly.

The distinction between contexts ensures that normal user operations remain appropriately scoped while still allowing the system to perform necessary administrative functions.

## File Operations

The drive system provides file storage and management capabilities with granular permission controls. Listing files through the API shows different results depending on context—in user context, only the user's files appear, while system context reveals all files within the bot's storage. Similarly, file uploads target the user's folder in user context but can write to any location in the bot's storage when operating in system context.

File deletion and sharing follow the same pattern. Users can delete and share their own files, while system context permits these operations on any file. The corresponding permissions are `files:read` for viewing and downloading, `files:write` for uploading and modifying, `files:delete` for removal, and `files:share` for granting access to others.

## Communication Capabilities

Email functionality provides access to messaging through the organization's mail system. Reading inbox contents and drafts requires the `email:read` permission and operates strictly in user context—there's no meaningful system context for reading another user's email. Sending messages requires `email:send` and can operate in either context, with user context messages appearing to come from the user and system context messages appearing to come from the bot.

Meeting integration enables video conferencing coordination. Viewing room information uses `meet:read`, with user context showing only visible rooms and system context revealing all rooms. Creating meetings requires `meet:create`, where user context establishes the creator as organizer while system context creates bot-organized meetings. Joining requires `meet:join` and inviting others requires `meet:invite`, with system context allowing invitations to any meeting regardless of ownership.

Calendar operations manage scheduling and appointments. Reading events with `calendar:read` shows user events in user context or bot calendar events in system context. Creating events requires `calendar:write` and targets the appropriate calendar based on context. Booking appointments with `calendar:book` makes the user an attendee in user context or establishes the bot as organizer in system context.

Task management follows similar patterns. The `tasks:read` permission shows user tasks in user context or all tasks in system context. Creating and modifying tasks with `tasks:write` assigns tasks appropriately based on context. Completing tasks with `tasks:complete` allows users to mark their own tasks complete or, in system context, to complete any task.

## Administrative Functions

Administrative endpoints provide system management capabilities reserved for privileged users. Managing users requires `admin:users`, managing bot configurations requires `admin:bots`, modifying system configuration requires `admin:config`, and accessing monitoring data requires `admin:monitor`. All administrative operations execute in system context and require explicit administrative privileges.

These elevated permissions should be granted sparingly, typically only to IT staff responsible for system operation. The audit system tracks all administrative actions to maintain accountability.

## Permission Definitions

The permission system defines specific capabilities organized by functional area. Core permissions govern fundamental platform features: `chat:read` allows viewing conversation history, `chat:write` enables sending messages, and the file permissions control document management as described above.

Communication permissions extend to the various messaging channels: email read and send capabilities, meeting room operations, and calendar management. Productivity permissions cover task management operations.

Administrative permissions form a separate category with broader impact: `admin:users` for user management, `admin:groups` for group administration, `admin:bots` for bot configuration, `admin:config` for system settings, `admin:monitor` for accessing operational metrics, and `admin:backup` for data protection operations.

## Default Group Configuration

General Bots creates several default groups during initialization, each designed for common organizational roles.

The Administrators group receives all permissions, including the complete set of administrative capabilities. Members of this group can perform any operation in the system. This group should contain only trusted IT personnel responsible for platform operation.

The Managers group provides access to productivity features plus basic monitoring capabilities. Managers can fully utilize chat, files including sharing, email, meetings, calendar, and tasks. They can also view monitoring data to understand system usage but cannot modify system configuration or manage users.

The Users group establishes standard access for regular employees. Users can participate in chat, work with files without sharing capabilities, read and send email, view and join meetings, manage their calendars, and handle their tasks. This permission set enables full participation in daily work without administrative capabilities.

The Guests group provides minimal access for anonymous or temporary users. Guests can only participate in chat, without access to any other system features. This restricted access suits scenarios where external parties need limited interaction with bots.

## Permission Configuration

Configuring permissions involves coordinating settings between Zitadel and the General Bots configuration.

In Zitadel, administrators access the admin console and navigate to Organization settings, then to Roles. Here they create roles that correspond to the permissions defined in General Bots. These roles are then assigned to groups, and users are added to appropriate groups based on their organizational responsibilities.

The config.csv file for each bot can map Zitadel roles to General Bots permissions. The permission mapping entries define which local permissions correspond to each Zitadel role. The default anonymous permission setting establishes what capabilities unauthenticated users receive.

## Anonymous Access Considerations

The chat interface supports anonymous users who haven't authenticated, though with significant restrictions. Anonymous users can chat with the default bot only, using a session that exists solely on the server. They cannot access conversation history, the drive, email, tasks, meetings, or any settings. Essentially, anonymous access provides a preview of bot capabilities without exposing organizational resources.

Organizations can customize the default anonymous permissions if they want to provide different capabilities to unauthenticated users, though most deployments restrict anonymous access to basic chat functionality.

## Permission Checking in Scripts

BASIC scripts can query user roles to implement conditional logic based on permissions. By retrieving the role from the session, scripts can present different options or perform different actions depending on the user's access level.

For example, a script might offer administrative functions only to users with the admin role, provide reporting features to managers, and present standard assistance to regular users. This capability allows bots to adapt their behavior to each user's organizational context.

## Audit Trail

All permission checks are logged, creating a comprehensive audit trail of access attempts. Administrators can query these logs through the admin API to review permission-related events. Each log entry captures the timestamp, user identifier, attempted action, accessed resource, result indicating whether access was allowed or denied, and when denied, the reason for denial.

This audit capability supports security reviews, compliance requirements, and troubleshooting access issues. Organizations with regulatory obligations can demonstrate that appropriate access controls are in place and functioning correctly.

## Related Documentation

For deeper understanding of the authentication and authorization system, the User Authentication chapter explains the login and session management processes. The User Context vs System Context chapter provides detailed exploration of how context affects API behavior. The Security Policy chapter establishes guidelines for secure platform operation. The API Endpoints chapter documents the full API surface including permission requirements for each endpoint.