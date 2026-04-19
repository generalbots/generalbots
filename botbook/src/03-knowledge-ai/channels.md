# Multi-Channel Support

This chapter describes how botserver enables bots to communicate with users across different platforms through its flexible multi-channel architecture. The design ensures that conversation logic remains consistent regardless of how users choose to interact, while still taking advantage of each channel's unique capabilities.

## Architectural Foundation

botserver abstracts communication methods through a channel adapter pattern that separates bot logic from platform-specific details. When a user sends a message, it flows from their chosen platform through a channel adapter that converts the platform-specific format into a common message structure. The bot processes this message, generates a response, and the adapter converts it back to the appropriate format for delivery.

This abstraction provides significant benefits for bot development. The same BASIC scripts work across all supported channels without modification. Conversation state persists even when users switch between platforms. New channels can be added without changing existing bot logic.

The primary channel is the web interface, which provides the richest interaction capabilities. WebSocket connections enable real-time messaging with streaming responses. Additional channels extend reach to users on their preferred platforms while maintaining consistent conversation experiences.

## The Web Channel

The web channel serves as the reference implementation and primary interaction method for most deployments. It leverages HTTP for initial page loads and WebSocket connections for real-time bidirectional communication.

Users interacting through the web channel benefit from rich formatting through Markdown support, clickable suggestion buttons that simplify common interactions, file upload and attachment capabilities, inline image display, typing indicators that show when the bot is processing, and theme customization that allows organizations to brand the interface.

The implementation handles WebSocket connection management, maintaining long-lived connections with heartbeat mechanisms to detect disconnections. When a connection drops, clients can automatically reconnect and resume their session without losing conversation context.

## Voice Interaction

When the voice feature is enabled, botserver supports spoken interaction through speech-to-text and text-to-speech processing. Voice conversations follow a continuous flow where the system listens for user speech, converts it to text, processes it through the same BASIC scripts used for text channels, and converts the response back to speech for playback.

This channel requires integration with speech services and is optional due to its additional infrastructure requirements. Organizations that enable voice interaction can serve users who prefer speaking to typing or who are in situations where hands-free operation is beneficial.

## Unified Session Management

All channels share a common session system, which is essential for maintaining coherent conversations across platform switches. When a user first interacts with a bot, the system creates a session that stores conversation context, user preferences, and any data accumulated during the interaction.

This session persists independently of the channel being used. A user could begin a conversation on the web interface from their desktop, continue it later on a mobile device, and the bot would have full context of previous exchanges. The session stores user identification information linked through authentication, ensuring that cross-channel continuity works correctly for logged-in users.

Session data includes conversation history, variables set during script execution, user preferences such as language settings, and references to any files or documents shared during the conversation.

## Message and Response Structures

The common message format bridges platform-specific protocols to the unified bot processing system. Each message carries the text content provided by the user, identifiers linking it to the user and session, the channel type indicating its origin, and a metadata field for channel-specific information that might be relevant to processing.

Responses follow a structured format that channel adapters interpret appropriately. Beyond the main content text, responses can include suggestion arrays that channels supporting quick replies render as buttons, a message type indicator distinguishing text from cards or media, streaming tokens for channels that support progressive response display, and completion flags indicating whether the response is final.

Channel adapters examine these response components and render them appropriately for their platform. A suggestion might become a clickable button on the web, a numbered list in a text-only channel, or ignored entirely in voice where such interaction patterns don't apply.

## Adaptive Bot Behavior

While the goal is channel-agnostic scripts, situations arise where bots benefit from knowing their communication context. Scripts can query the current channel and adapt their behavior accordingly, offering voice-appropriate prompts when speaking to users or visual elements when they're available.

Feature detection allows scripts to check whether the current channel supports specific capabilities before attempting to use them. Rather than checking the channel type directly, checking for feature support makes scripts more resilient to future channel additions that might have different capability combinations.

This adaptive capability should be used sparingly. Most bot logic should remain channel-agnostic, with adaptations limited to presentation concerns rather than core functionality.

## WebSocket Communication Protocol

The WebSocket protocol defines how clients and servers exchange messages over persistent connections. Clients initiate connections to the `/ws` endpoint, where the server creates or retrieves their session and establishes the bidirectional channel.

Messages from clients to the server carry a type field indicating the message kind, the content being sent, and the session identifier linking the message to an existing conversation. The server responds with structured messages including the response content, any suggestions to display, and flags indicating whether the response is complete or if more content will follow for streaming scenarios.

The protocol includes heartbeat messages to maintain connection liveness across network infrastructure that might otherwise terminate idle connections. Both client and server implementations should handle reconnection gracefully, allowing conversations to continue after temporary network interruptions.

## Expanding Channel Support

The architecture anticipates integration with additional platforms including WhatsApp Business API, Microsoft Teams, Slack, Telegram, Discord, and SMS gateways. While these channels aren't implemented in the current version, the adapter pattern provides a clear path for adding them.

Implementing a new channel involves creating an adapter that implements the standard interface for sending and receiving messages, handling the platform's specific authentication and webhook requirements, mapping between the platform's message format and the common structure, registering supported features accurately so scripts can adapt appropriately, and managing any platform-specific rate limits or constraints.

The separation of concerns in the adapter pattern means that new channels don't require changes to bot logic, session management, or the BASIC execution environment. They plug into the existing infrastructure at well-defined integration points.

## Practical Considerations

Several factors influence channel selection and implementation for production deployments. Feature availability varies significantly between channels, with web providing the richest interaction while text-only channels offer broader reach. Rich formatting and media support depend entirely on the destination platform's capabilities.

Network reliability affects real-time channels differently than store-and-forward systems like email or SMS. WebSocket connections require stable networks, while messaging platforms handle intermittent connectivity through their own infrastructure.

Authentication requirements differ between channels. The web channel integrates with the platform's standard OAuth flow, while messaging platforms typically use their own identity systems that must be mapped to General Bots users.

Rate limiting applies per channel and must be respected to maintain good standing with platform providers. Automated messages face stricter limits than user-initiated conversations on most platforms.

## Development Guidelines

Effective multi-channel bot development follows several principles. Writing channel-agnostic scripts as the default approach maximizes code reuse and simplifies maintenance. Using universal keywords like TALK and HEAR ensures scripts work everywhere without modification.

Testing across channels validates that the user experience remains coherent despite platform differences. What works well on web might need adjustment for voice or text-only channels. Identifying these differences during development prevents surprises in production.

Preserving session state carefully ensures that cross-channel continuity works correctly. Scripts should store important context in session variables rather than relying on channel-specific features that might not translate.

Monitoring channel metrics helps identify performance issues or user experience problems specific to particular platforms. Response times, error rates, and user satisfaction can vary significantly between channels.

## Summary

botserver's multi-channel architecture enables bots to reach users wherever they prefer to communicate while maintaining consistent conversation logic and state. The channel adapter pattern isolates platform-specific concerns from bot development, allowing the same scripts to work across current channels and future integrations. This design philosophy prioritizes developer productivity and user experience across an expanding communication landscape.