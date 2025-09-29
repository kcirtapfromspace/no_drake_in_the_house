# iOS Shortcuts Integration

This directory contains iOS Shortcuts for the Music Streaming Blocklist Manager.

## Available Shortcuts

1. **Add Artist to DNP List** - Quick shortcut to add an artist to your Do Not Play list
2. **Remove Artist from DNP List** - Remove an artist from your DNP list
3. **Check Enforcement Status** - Get current enforcement status across connected services
4. **Skip Current Track** - Skip the currently playing track and optionally add to DNP list
5. **Voice DNP Management** - Siri-enabled voice commands for DNP list management

## Installation

1. Download the `.shortcut` files from this directory
2. Open them on your iOS device
3. Follow the prompts to install and configure
4. Grant necessary permissions for media controls and network access

## Configuration

Before using the shortcuts, you'll need to:
1. Set your API base URL (default: https://api.nodrakeinthe.house)
2. Authenticate with your account to get an API token
3. Configure which streaming services you want to control

## Siri Integration

All shortcuts support Siri voice commands:
- "Hey Siri, add [artist name] to my DNP list"
- "Hey Siri, check my enforcement status"
- "Hey Siri, skip this song and block the artist"