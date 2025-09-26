# Kiro Browser Extension

A browser extension for blocking unwanted artists across music streaming platforms.

## Features

- **Multi-Platform Support**: Works with Spotify, YouTube Music, Apple Music, and Tidal
- **Content Filtering**: Hides blocked artists in playlists, search results, and recommendations
- **Auto-Skip**: Automatically skips tracks from blocked artists
- **Quick Actions**: Add/remove artists from blocklist with right-click context menus
- **Sync Support**: Synchronizes with Kiro server for cross-device consistency
- **Offline Mode**: Works offline with cached blocklist data

## Installation

### Development Installation

1. Clone the repository and navigate to the extension directory
2. Open Chrome and go to `chrome://extensions/`
3. Enable "Developer mode" in the top right
4. Click "Load unpacked" and select the `extension` directory
5. The extension should now appear in your extensions list

### Production Installation

The extension will be available on the Chrome Web Store once published.

## Setup

1. Click the Kiro extension icon in your browser toolbar
2. If you have a Kiro account, enter your authentication token in settings
3. The extension will automatically sync your blocklist
4. Visit supported streaming sites to see the extension in action

## Supported Platforms

| Platform | Content Hiding | Auto-Skip | Artist Detection | Notes |
|----------|----------------|-----------|------------------|-------|
| Spotify | ✅ Full | ✅ Full | ✅ Excellent | Best support |
| YouTube Music | ✅ Full | ✅ Full | ✅ Good | Web-based approach |
| Apple Music | ✅ Full | ✅ Limited | ✅ Good | Limited API access |
| Tidal | ✅ Full | ✅ Limited | ✅ Good | Best-effort support |

## Usage

### Adding Artists to Blocklist

1. **From Extension Popup**: Click the extension icon and use "Block Current Artist"
2. **Context Menu**: Right-click on any artist name or link
3. **Quick Actions**: Use the overlay controls when blocked content is detected
4. **Web Dashboard**: Manage your full blocklist at the Kiro web app

### Managing Blocked Content

- **Hidden Content**: Blocked artists appear grayed out with "Hidden by Kiro" badges
- **Override Options**: Click badges to reveal "Play Once" or "Unblock" options
- **Auto-Skip**: Tracks are automatically skipped with notifications
- **Manual Control**: Disable auto-skip in extension settings if preferred

### Syncing with Server

1. Open extension settings (right-click extension icon → Options)
2. Enter your Kiro server URL (default: http://localhost:3000)
3. Add your authentication token from the web dashboard
4. Click "Connect" to test the connection
5. The extension will sync automatically every 5 minutes

## Settings

Access settings by right-clicking the extension icon and selecting "Options":

- **Auto-Skip**: Automatically skip blocked tracks
- **Show Notifications**: Display skip notifications
- **Hide Content**: Visually hide blocked artists
- **Sync Interval**: How often to sync with server (1-60 minutes)
- **Server URL**: Your Kiro server endpoint
- **Auth Token**: Your authentication token

## Privacy

- The extension only stores your blocklist and basic usage statistics
- No personal information or browsing history is collected
- All data is stored locally in your browser
- Server sync is optional and uses encrypted connections
- You can export/import your data at any time

## Troubleshooting

### Extension Not Working

1. Check that the extension is enabled in `chrome://extensions/`
2. Refresh the streaming service page
3. Check browser console for error messages
4. Try disabling and re-enabling the extension

### Content Not Being Hidden

1. Verify the artist is in your blocklist (extension popup → View Blocklist)
2. Check that "Hide Content" is enabled in settings
3. Some content may take a moment to be detected and hidden
4. Try refreshing the page

### Auto-Skip Not Working

1. Ensure "Auto-Skip" is enabled in settings
2. Check that the streaming service supports media controls
3. Some platforms have limited auto-skip capabilities
4. Manual skip buttons should still work

### Sync Issues

1. Verify your server URL is correct
2. Check that your auth token is valid
3. Ensure the Kiro server is running and accessible
4. Check network connectivity

## Development

### File Structure

```
extension/
├── manifest.json              # Extension manifest
├── background.js             # Service worker
├── popup.html/js            # Extension popup
├── settings.html/js         # Settings page
├── content-scripts/         # Platform-specific scripts
│   ├── base-content-script.js
│   ├── spotify.js
│   ├── youtube-music.js
│   ├── apple-music.js
│   └── tidal.js
├── ui/                      # UI components
│   └── overlay.js
└── icons/                   # Extension icons
```

### Adding New Platforms

1. Create a new content script in `content-scripts/`
2. Extend `BaseContentScript` class
3. Implement platform-specific artist detection
4. Add content script to `manifest.json`
5. Test thoroughly on the target platform

### Building for Production

1. Update version in `manifest.json`
2. Optimize and minify JavaScript files
3. Add proper extension icons
4. Test on multiple browsers/platforms
5. Package as ZIP for Chrome Web Store

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support, please:
1. Check this README and troubleshooting section
2. Search existing GitHub issues
3. Create a new issue with detailed information
4. Contact the development team

## Changelog

### Version 1.0.0
- Initial release
- Support for Spotify, YouTube Music, Apple Music, Tidal
- Content hiding and auto-skip functionality
- Server sync capabilities
- Settings and data management