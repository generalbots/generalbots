# API Endpoints

This chapter provides a comprehensive reference for the API endpoints exposed by General Bots. The platform offers RESTful endpoints for authentication, session management, user operations, and bot interactions, as well as WebSocket connections for real-time communication.

## Authentication Endpoints

Authentication in General Bots is delegated to the Directory Service, which implements industry-standard OAuth2 and OpenID Connect protocols. The authentication endpoints primarily serve as integration points with this external identity provider.

### OAuth Login

The login process begins when a client application directs the user to the `/auth/login` endpoint using a GET request. This endpoint does not require any request body or authentication headers, as its purpose is to initiate the OAuth2 flow. Upon receiving this request, the server generates appropriate OAuth2 parameters and redirects the user's browser to the Zitadel login page, where they can enter their credentials securely within the identity provider's domain.

### OAuth Callback

After successful authentication with the Directory Service, the user's browser is redirected back to `/auth/callback` with authorization parameters. This GET endpoint expects two query parameters: a `code` parameter containing the authorization code issued by the Directory Service, and a `state` parameter that serves as a CSRF protection mechanism to ensure the callback corresponds to a legitimate login attempt.

When the callback is processed successfully, the server exchanges the authorization code for access tokens, creates a local session, sets a session cookie in the response, and redirects the user to the main application interface. This seamless flow means users typically don't notice the redirect chain happening in the background.

### Logout

To terminate a session, clients send a POST request to `/auth/logout`. This endpoint requires the current session token in the Authorization header using the Bearer scheme. The server invalidates the session both locally and with the Directory Service, returning a JSON response confirming successful logout. After logout, the session token becomes invalid and cannot be used for further requests.

### Session Validation

The `/auth/validate` endpoint allows clients to verify whether their current session token remains valid without performing any other operation. By sending a GET request with the session token in the Authorization header, clients receive a JSON response indicating whether the token is valid, the associated user identifier, and the session's expiration timestamp. This endpoint is particularly useful for single-page applications that need to check session status before making other API calls.

## Session Management

Session management endpoints provide control over the user's active sessions and their associations with bots.

### Current Session Information

Clients can retrieve information about their current session by sending a GET request to `/api/session`. The response includes the session identifier, the user's identifier, the currently selected bot identifier if any, and timestamps indicating when the session was created and when it will expire. This information helps applications understand the current authentication context and present appropriate interface elements.

### Creating Bot Sessions

When a user wants to interact with a specific bot, the application creates a bot session by sending a POST request to `/api/session/create`. The request body contains a JSON object with the target bot's identifier. If the user has permission to access the requested bot, the server creates a new session linking the user to that bot and returns the session details including its identifier, the associated bot identifier, and the session's active status.

This separation between authentication sessions and bot sessions allows users to maintain their login while switching between different bots without requiring re-authentication.

### Terminating Sessions

To end a specific session, clients send a DELETE request to `/api/session/:id`, where the path parameter identifies the session to terminate. The server validates that the requester has permission to terminate the specified session, typically by verifying they own it, and then invalidates it. The response confirms whether the termination succeeded.

## User Management

User endpoints provide access to profile information and allow limited profile modifications.

### Retrieving User Information

The `/api/users/me` endpoint responds to GET requests with the current user's profile information. This includes their unique identifier, username, email address, and account creation timestamp. Since user data is managed in the Directory Service, this endpoint essentially proxies information from that system into a format convenient for the application.

### Profile Updates

Users can update certain profile fields by sending a PUT request to `/api/users/me` with a JSON body containing the fields to modify. Supported fields typically include email address, first name, and last name. It's important to note that these updates are actually propagated to the Directory Service, which serves as the authoritative source for user information. The endpoint validates the requested changes and forwards them to Zitadel for persistence.

## Bot Interaction

Real-time communication with bots occurs primarily through WebSocket connections, though REST endpoints exist for bot discovery.

### WebSocket Communication

The primary channel for bot interaction is the WebSocket endpoint at `/ws`. After establishing a connection, clients send JSON-formatted messages containing a message type, the content of the message, and the session identifier. The server processes these messages, routes them to the appropriate bot, and sends responses back through the same WebSocket connection.

This real-time bidirectional communication enables responsive conversational experiences without the overhead of repeated HTTP connections. The WebSocket connection maintains state throughout the conversation, allowing for context-aware responses.

### Bot Discovery

Users discover available bots by sending a GET request to `/api/bots`. The response contains an array of bot objects, each including the bot's identifier, display name, description, and current operational status. Only bots that the authenticated user has permission to access appear in this list, ensuring users see a curated view appropriate to their organizational role and permissions.

## Administrative Endpoints

Administrative endpoints provide system management capabilities for users with appropriate privileges. The system status endpoint at `/api/admin/system/status` returns health information about the various system components. The metrics endpoint at `/api/admin/system/metrics` provides operational statistics useful for monitoring and capacity planning. Both endpoints require administrative privileges, which are validated against the user's roles in the Directory Service.

## Group Management

Group management endpoints support the organization's permission structure. The `/api/groups/create` endpoint accepts POST requests to establish new groups. The `/api/groups/list` endpoint returns all groups visible to the requesting user. Individual group membership can be queried through `/api/groups/:id/members`. These endpoints work in conjunction with the Directory Service to maintain consistent group definitions across the platform.

## Rate Limiting

To ensure fair resource allocation and protect against abuse, all API endpoints implement rate limiting. Public endpoints, including the health check, allow 60 requests per hour from unauthenticated clients. Authenticated users can make up to 1000 requests per hour across all endpoints. Administrative users receive a higher limit of 5000 requests per hour to accommodate their management responsibilities.

Rate limit information is communicated through response headers. The `X-RateLimit-Limit` header indicates the maximum requests allowed in the current window, `X-RateLimit-Remaining` shows how many requests remain, and `X-RateLimit-Reset` provides a Unix timestamp indicating when the limit resets. Applications should monitor these headers and implement appropriate backoff strategies when approaching limits.

## Error Handling

All API endpoints return errors in a consistent JSON format. The response body contains an error object with a machine-readable code, a human-readable message, and an optional details object providing additional context. Common error codes include `UNAUTHORIZED` for missing or invalid authentication, `FORBIDDEN` when the user lacks required permissions, `NOT_FOUND` for requests targeting non-existent resources, `RATE_LIMITED` when request quotas are exceeded, and `SERVER_ERROR` for internal failures.

Clients should implement error handling that examines the error code to determine appropriate recovery actions. Authentication errors might prompt a re-login flow, while rate limiting errors should trigger request throttling.

## Cross-Origin Resource Sharing

The API supports Cross-Origin Resource Sharing (CORS) to enable browser-based applications hosted on different domains. In development environments, the server accepts requests from any origin. Production deployments should configure specific allowed origins to prevent unauthorized cross-domain access. The allowed methods include GET, POST, PUT, DELETE, and OPTIONS, with Content-Type and Authorization as permitted headers.

## Health Monitoring

The `/health` endpoint provides a simple way to verify the server is operational. Unlike other endpoints, this one requires no authentication, making it suitable for external monitoring systems and load balancer health checks. The response includes a status indicator and a timestamp, providing basic confirmation that the server can process requests.

## Implementation Status

The current implementation provides full support for WebSocket communication, administrative endpoints, group management, and health checking. OAuth authentication flows through the Directory Service are functional but continue to evolve. Session management endpoints work for basic scenarios with ongoing enhancements planned. Some user profile endpoints and direct REST messaging capabilities remain under development, with batch operations planned for future releases.

## Security Considerations

Several security practices should guide API usage. With the exception of the health endpoint, all API calls require valid authentication. Administrative operations additionally verify that the requester holds appropriate roles within the Directory Service. Session tokens must be treated as secrets, stored securely on clients, and never logged or exposed. Production deployments must use HTTPS to encrypt all API traffic. Applications performing state-changing operations should implement CSRF protection through the state parameter and appropriate token validation.

## Recommended Practices

Effective API integration follows several patterns. Always include the session token in the Authorization header for authenticated requests. Implement graceful handling of token expiration by detecting authentication errors and prompting re-login when necessary. Use exponential backoff for retry logic, starting with short delays and increasing them progressively for repeated failures. Cache responses where appropriate to reduce server load and improve application responsiveness. Prefer WebSocket connections for conversational interactions where real-time response is important. Monitor rate limit headers proactively to avoid hitting limits during normal operation.