# General Bots Chrome Extension

A professional-grade Chrome extension developed by [pragmatismo.com.br](https://pragmatismo.com.br) that enhances WhatsApp Web with server-side message processing capabilities and UI improvements.

## Features

- Message Interception: Captures messages before they're sent
- Server Processing: Sends message content to your server for processing
- Message Replacement: Updates the message with processed content before sending
- UI Enhancement: Option to hide the contact list for more chat space
- User-friendly Settings: Simple configuration through the extension popup

## Installation

### Developer Mode Installation

1. Clone or download this repository
2. Open Chrome and navigate to `chrome://extensions/`
3. Enable "Developer mode" in the top-right corner
4. Click "Load unpacked" and select the extension directory

### Chrome Web Store Installation

(Coming soon)

## Configuration

1. Click the General Bots icon in your Chrome toolbar
2. Enter your processing server URL
3. Toggle message processing on/off
4. Toggle contact list visibility

## Server API Requirements

Your server endpoint should:

1. Accept POST requests with JSON payload: `{ "text": "message content", "timestamp": 1621234567890 }`
2. Return JSON response: `{ "processedText": "updated message content" }`

## License

This project is licensed under the [GNU Affero General Public License](LICENSE) - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Contact

For support or questions, please contact [pragmatismo.com.br](https://pragmatismo.com.br).
