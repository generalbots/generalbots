/**
 * Message Type Constants
 * Defines the different types of messages in the bot system
 * These values must match the server-side MessageType enum in Rust
 */

const MessageType = {
    /** Regular message from external systems (WhatsApp, Instagram, etc.) */
    EXTERNAL: 0,

    /** User message from web interface */
    USER: 1,

    /** Bot response (can be regular content or event) */
    BOT_RESPONSE: 2,

    /** Continue interrupted response */
    CONTINUE: 3,

    /** Suggestion or command message */
    SUGGESTION: 4,

    /** Context change notification */
    CONTEXT_CHANGE: 5
};

/**
 * Get the name of a message type
 * @param {number} type - The message type number
 * @returns {string} The name of the message type
 */
function getMessageTypeName(type) {
    const names = {
        0: 'EXTERNAL',
        1: 'USER',
        2: 'BOT_RESPONSE',
        3: 'CONTINUE',
        4: 'SUGGESTION',
        5: 'CONTEXT_CHANGE'
    };
    return names[type] || 'UNKNOWN';
}

/**
 * Check if a message is a bot response
 * @param {Object} message - The message object
 * @returns {boolean} True if the message is a bot response
 */
function isBotResponse(message) {
    return message && message.message_type === MessageType.BOT_RESPONSE;
}

/**
 * Check if a message is a user message
 * @param {Object} message - The message object
 * @returns {boolean} True if the message is from a user
 */
function isUserMessage(message) {
    return message && message.message_type === MessageType.USER;
}

/**
 * Check if a message is a context change
 * @param {Object} message - The message object
 * @returns {boolean} True if the message is a context change
 */
function isContextChange(message) {
    return message && message.message_type === MessageType.CONTEXT_CHANGE;
}

/**
 * Check if a message is a suggestion
 * @param {Object} message - The message object
 * @returns {boolean} True if the message is a suggestion
 */
function isSuggestion(message) {
    return message && message.message_type === MessageType.SUGGESTION;
}

// Export for use in other modules (if using modules)
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        MessageType,
        getMessageTypeName,
        isBotResponse,
        isUserMessage,
        isContextChange,
        isSuggestion
    };
}

// Also make available globally for non-module scripts
if (typeof window !== 'undefined') {
    window.MessageType = MessageType;
    window.getMessageTypeName = getMessageTypeName;
    window.isBotResponse = isBotResponse;
    window.isUserMessage = isUserMessage;
    window.isContextChange = isContextChange;
    window.isSuggestion = isSuggestion;
}
