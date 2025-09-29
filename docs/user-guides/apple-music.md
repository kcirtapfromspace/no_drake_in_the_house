# Apple Music Integration Guide

Apple Music integration provides good library management capabilities with some limitations due to platform restrictions.

## 🎯 What You Can Do

### ✅ Fully Supported Features

- **Library Management**: Remove songs and albums from your personal library
- **Artist Following**: Unfollow blocked artists
- **Web Interface**: Hide content and auto-skip tracks in Apple Music web player
- **Featured Artist Detection**: Identify collaborations and featured appearances
- **Batch Operations**: Process libraries efficiently within API limits

### ⚠️ Limited Features

- **Playlist Modifications**: Limited to personal playlists only
- **Collaborative Playlists**: Cannot modify shared playlists
- **Radio Stations**: Cannot directly influence personal radio recommendations
- **Smart Playlists**: Cannot modify automatically generated playlists

### ❌ Unsupported Features

- **Third-party Playlists**: Cannot modify playlists created by Apple or other users
- **Apple Music 1 Content**: Cannot block content from live radio shows
- **Shared Family Playlists**: Limited access to family sharing playlists

## 🔗 Connecting Your Apple Music Account

### Step 1: MusicKit Authorization

1. Go to **Settings** → **Connected Services**
2. Click **Connect Apple Music**
3. You'll see the MusicKit authorization dialog
4. Review the requested permissions:
   - **Media Library**: Read and modify your music library
   - **User Profile**: Access your Apple Music profile
   - **Playlists**: Read and modify your personal playlists
5. Click **Allow** to authorize

### Step 2: Developer Token Setup

Apple Music requires both user authorization and developer tokens:
- **User Token**: Authorizes access to your personal library
- **Developer Token**: Enables API access (managed automatically)
- **Token Rotation**: Tokens are refreshed automatically every 6 months

### Step 3: Verify Connection

After authorization, you should see:
- ✅ **Apple Music Connected** with your Apple ID
- **Subscription Status**: Active Apple Music subscription required
- **Last sync**: Timestamp of last successful sync
- **Token Status**: Valid user and developer tokens

## 🎵 Enforcement Capabilities

### Library Enforcement

**Personal Library**
- Removes songs where blocked artists are primary artists
- Handles featured artists and collaborations
- Preserves songs with only background vocals (configurable)
- Maintains library organization and metadata

**Album Management**
- Removes entire albums by blocked artists
- Handles compilation albums intelligently
- Preserves albums with mixed artists (configurable)

### Playlist Enforcement

**Personal Playlists**
- Removes tracks from playlists you created
- Maintains playlist order and metadata
- Handles custom playlist artwork and descriptions

**Limitations**
- Cannot modify collaborative playlists
- Cannot modify Apple-curated playlists
- Cannot modify smart playlists (auto-generated)

### Artist Management

- Unfollows blocked artists from your profile
- Removes from "Artists you follow" section
- May reduce their appearance in For You recommendations

## 🔧 Configuration Options

### Enforcement Aggressiveness

**Conservative**
- Only removes primary artist appearances
- Skips collaborative content
- Preserves featured appearances in compilations

**Moderate** (Recommended)
- Removes primary and featured appearances
- Processes personal playlists only
- Handles most collaboration types

**Aggressive**
- Removes all appearances including background vocals
- Uses advanced detection for uncredited appearances
- Processes all accessible content

### Apple Music Specific Settings

**Library Handling**
```
☑️ Remove songs from library
☑️ Remove albums from library
☑️ Unfollow artists
☐ Remove from recently played (not supported)
```

**Playlist Options**
```
☑️ Process personal playlists
☐ Skip collaborative playlists (recommended)
☑️ Preserve playlist metadata
☑️ Maintain track order
```

**Detection Settings**
```
☑️ Block featuring artists
☑️ Block collaborations
☐ Block songwriter credits (limited data)
☐ Block producer credits (limited data)
```

## 🌐 Browser Extension Features

### Apple Music Web Player

The browser extension works with music.apple.com:

**Content Filtering**
- Hides artist tiles in browse sections
- Dims blocked tracks in playlists and albums
- Shows "Hidden by Kiro" badges on filtered content

**Auto-Skip Functionality**
- Automatically skips blocked tracks during playback
- Configurable skip delay (0-5 seconds)
- Shows notification with skip reason

**Override Controls**
- Right-click hidden content for options:
  - **Play Once**: Allow this track temporarily
  - **Remove from DNP**: Remove artist from blocklist
  - **Report Issue**: Report detection problems

### Installation & Setup

1. Install the browser extension
2. Navigate to music.apple.com
3. Log in to your Apple Music account
4. The extension will automatically detect and sync your DNP list
5. Refresh the page to see filtering in effect

## 📊 Enforcement Planning

### Dry Run Preview

Apple Music enforcement planning shows:

```
📊 Apple Music Enforcement Preview

Personal Library: 67 items to remove
├── Songs: 52 tracks
├── Albums: 15 albums
└── Featured appearances: 8 tracks

Personal Playlists: 5 playlists to modify
├── "My Favorites": 12 tracks to remove
├── "Workout Mix": 8 tracks to remove
├── "Road Trip": 5 tracks to remove
└── Warnings: 2 collaborative playlists skipped

Following: 2 artists to unfollow

⚠️ Limitations:
- 3 collaborative playlists will be skipped
- Smart playlists cannot be modified
- Recently played history cannot be cleared

Estimated time: 1 minute 45 seconds
```

### Execution Process

1. **Library Phase**: Removes songs and albums from personal library
2. **Playlist Phase**: Processes personal playlists only
3. **Following Phase**: Unfollows blocked artists
4. **Verification Phase**: Confirms all changes were applied

## 🔄 Rollback & Undo

### Rollback Capabilities

✅ **Reversible Actions**
- Re-add songs to library (if still available in Apple Music catalog)
- Re-add albums to library
- Re-add tracks to personal playlists
- Re-follow artists

❌ **Irreversible Actions**
- Changes to collaborative playlists (limited access)
- Modifications to smart playlists (auto-generated)
- Apple Music recommendation algorithm changes

### Rollback Process

1. Go to **History** → **Enforcement History**
2. Find the Apple Music execution
3. Click **Rollback** (available for 7 days)
4. Choose rollback scope:
   - **Library Only**: Restore songs and albums
   - **Playlists Only**: Restore playlist modifications
   - **Full Rollback**: Restore all changes

## ⚡ Performance & Rate Limits

### Apple Music API Limits

- **Rate Limit**: 1000 requests per hour per developer token
- **User Rate Limit**: 100 requests per minute per user
- **Batch Size**: Up to 25 items per request
- **Token Refresh**: Every 6 months automatically

### Optimization Features

**Efficient Batching**
- Groups library operations for maximum efficiency
- Processes playlists in order of size (smallest first)
- Uses parallel requests where possible

**Token Management**
- Automatically refreshes user tokens
- Handles developer token rotation
- Graceful fallback for token issues

## 🔍 Troubleshooting

### Common Issues

**Subscription Required**
```
❌ "Apple Music subscription required"
```
**Solution**: Ensure you have an active Apple Music subscription

**Token Expired**
```
❌ "User token has expired"
```
**Solution**: Go to Settings → Connected Services → Reconnect Apple Music

**Limited Playlist Access**
```
⚠️ "Cannot modify collaborative playlist"
```
**Solution**: This is expected behavior - only personal playlists can be modified

**Missing Songs After Enforcement**
```
❌ "Songs not found in library after removal"
```
**Solution**: Check if songs are still available in Apple Music catalog

### Advanced Troubleshooting

**Web Player Not Filtering**
1. Ensure you're logged in to music.apple.com
2. Refresh the page after connecting your account
3. Check browser extension permissions
4. Clear browser cache and cookies for music.apple.com

**Incomplete Enforcement**
1. Check for Apple Music service outages
2. Verify your subscription is active
3. Look for rate limiting messages in the execution log
4. Try running enforcement during off-peak hours

## 📈 Best Practices

### Before Enforcement

1. **Verify Subscription**: Ensure Apple Music subscription is active
2. **Backup Playlists**: Export important playlists as backup
3. **Test with Small Lists**: Start with a few artists to test functionality
4. **Check Collaborative Playlists**: Note which playlists are collaborative

### During Enforcement

1. **Stay Connected**: Keep your browser tab open during execution
2. **Monitor Progress**: Watch for any error messages or warnings
3. **Don't Switch Accounts**: Avoid logging out of Apple Music during enforcement

### After Enforcement

1. **Verify Changes**: Check your library and playlists in the Apple Music app
2. **Test Web Player**: Ensure browser extension filtering is working
3. **Review Skipped Items**: Check why certain items were skipped
4. **Update Settings**: Adjust configuration based on results

## 🎯 Pro Tips

### Maximizing Apple Music Integration

**Library Organization**
- Use Apple Music's built-in organization features
- Create specific playlists for different enforcement levels
- Use the "Love" feature to protect important tracks

**Cross-Platform Sync**
- Changes sync across all your Apple devices
- Web player changes reflect in iOS/macOS apps
- iCloud Music Library must be enabled

**Family Sharing Considerations**
- Family playlists have limited modification access
- Individual family member libraries are separate
- Shared purchases are not affected by enforcement

### Workflow Optimization

**Regular Maintenance**
1. Run enforcement monthly for best results
2. Review and update DNP lists regularly
3. Check for new collaborative playlists
4. Monitor Apple Music catalog changes

**Integration with iOS**
- Use iOS Shortcuts for quick DNP management
- Set up Siri commands for common operations
- Use the iOS widget to monitor enforcement status

---

## Apple Music Limitations

### Platform Restrictions

Apple Music's API has inherent limitations:

**Collaborative Content**
- Cannot modify playlists shared with others
- Limited access to family sharing content
- Cannot change collaborative playlist settings

**Generated Content**
- Smart playlists are read-only
- Cannot modify Apple-curated playlists
- Radio recommendations have limited control

**Metadata Limitations**
- Less detailed collaboration information than Spotify
- Limited songwriter and producer credits
- Fewer external ID mappings available

### Workarounds

**For Collaborative Playlists**
1. Create personal copies of important collaborative playlists
2. Use the web extension for visual filtering
3. Manually remove tracks when possible

**For Smart Playlists**
1. Create manual playlists with similar criteria
2. Use the web extension to hide unwanted content
3. Regularly review and manually curate

---

## Need Help?

- **Apple Music API Status**: Check [Apple Developer System Status](https://developer.apple.com/system-status/)
- **Subscription Issues**: Contact [Apple Support](https://support.apple.com/apple-music)
- **Integration Problems**: Email [support@nodrakeinthe.house](mailto:support@nodrakeinthe.house)
- **Feature Requests**: Visit our [community forum](https://community.nodrakeinthe.house)