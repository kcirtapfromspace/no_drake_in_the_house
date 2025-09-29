# Android Integration

This directory contains Android integration components for the Music Streaming Blocklist Manager.

## Components

1. **Android Intents** - Deep linking and inter-app communication
2. **Tasker Integration** - Automation support for power users
3. **Quick Settings Tile** - Quick access to enforcement status
4. **Notification Controls** - Skip and block actions from notifications

## Installation

### APK Installation
1. Download the latest APK from releases
2. Enable "Install from Unknown Sources" in Android settings
3. Install the APK
4. Grant necessary permissions (notification access, media controls)

### Tasker Integration
1. Install Tasker from Google Play Store
2. Import the provided Tasker profiles from `tasker/` directory
3. Configure API credentials in Tasker variables
4. Enable the profiles

## Configuration

### API Setup
1. Open the app and go to Settings
2. Enter your API base URL (default: https://api.nodrakeinthe.house)
3. Authenticate with your account
4. Grant media control permissions when prompted

### Quick Settings Tile
1. Pull down notification shade
2. Tap "Edit" or pencil icon
3. Drag "DNP Status" tile to active tiles
4. Tap "Done"

## Permissions Required

- `BIND_NOTIFICATION_LISTENER_SERVICE` - For media session monitoring
- `MODIFY_AUDIO_SETTINGS` - For skip functionality
- `INTERNET` - For API communication
- `WAKE_LOCK` - For background processing
- `RECEIVE_BOOT_COMPLETED` - For auto-start functionality