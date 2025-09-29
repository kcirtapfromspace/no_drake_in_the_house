# Spotify Integration Guide

Spotify offers the most comprehensive integration with full API support for library management, playlist operations, and advanced features.

## 🎯 What You Can Do

### ✅ Fully Supported Features

- **Library Management**: Remove liked songs and saved albums
- **Playlist Operations**: Remove tracks from personal and collaborative playlists
- **Artist Management**: Unfollow blocked artists
- **Web Interface**: Hide content and auto-skip tracks in Spotify Web Player
- **Advanced Detection**: Identify featured artists and collaborations
- **Batch Operations**: Process large libraries efficiently with rate limiting

### ⚠️ Limited Features

- **Radio Seed Filtering**: Can influence but not completely control radio recommendations
- **Discover Weekly**: Cannot directly modify algorithmic playlists (they regenerate)

## 🔗 Connecting Your Spotify Account

### Step 1: Authorize Connection

1. Go to **Settings** → **Connected Services**
2. Click **Connect Spotify**
3. You'll be redirected to Spotify's authorization page
4. Review the requested permissions:
   - **Read access**: View your library, playlists, and followed artists
   - **Write access**: Modify your library and playlists
   - **User info**: Access your profile information
5. Click **Agree** to authorize

### Step 2: Verify Connection

After authorization, you should see:
- ✅ **Spotify Connected** with your username
- **Last sync**: Timestamp of connection
- **Health status**: Connection health indicator

### Required Permissions

The integration requests these Spotify scopes:
- `user-library-read` - View your saved tracks and albums
- `user-library-modify` - Remove tracks and albums from your library
- `playlist-read-private` - Access your private playlists
- `playlist-modify-private` - Modify your private playlists
- `playlist-modify-public` - Modify your public playlists
- `user-follow-read` - View artists you follow
- `user-follow-modify` - Unfollow artists

## 🎵 Enforcement Capabilities

### Library Enforcement

**Liked Songs**
- Removes tracks where blocked artists are primary artists
- Detects and removes collaborations (when enabled)
- Identifies featured artists in track metadata

**Saved Albums**
- Removes entire albums by blocked artists
- Handles compilation albums with mixed artists
- Preserves albums with only featured appearances (configurable)

### Playlist Enforcement

**Personal Playlists**
- Removes individual tracks by blocked artists
- Uses delta removal to minimize API calls
- Maintains playlist order and metadata

**Collaborative Playlists**
- Can remove tracks you added
- Cannot remove tracks added by other users
- Shows warnings for restricted operations

**Generated Playlists**
- Cannot modify Spotify-generated playlists (Discover Weekly, Release Radar)
- These playlists regenerate automatically

### Artist Following

- Unfollows blocked artists from your profile
- Removes from "Artists you follow" section
- May reduce their appearance in recommendations

## 🔧 Configuration Options

### Enforcement Aggressiveness

**Conservative**
- Only removes primary artist appearances
- Skips collaborative playlists
- Preserves featured appearances

**Moderate** (Recommended)
- Removes primary and featured appearances
- Processes collaborative playlists with warnings
- Handles most collaboration types

**Aggressive**
- Removes all appearances including background vocals
- Processes all playlists regardless of ownership
- Uses advanced detection algorithms

### Advanced Settings

**Collaboration Detection**
```
☑️ Block featuring artists (Artist feat. Blocked)
☑️ Block collaborations (Artist & Blocked Artist)
☐ Block songwriter-only credits
☐ Block producer credits
```

**Playlist Handling**
```
☑️ Process personal playlists
☑️ Process collaborative playlists (with warnings)
☐ Skip playlists with "mix" in the name
☑️ Preserve playlist order when possible
```

## 🌐 Browser Extension Features

### Web Player Integration

The browser extension enhances your Spotify Web Player experience:

**Content Hiding**
- Hides artist tiles in browse sections
- Dims blocked tracks in playlists
- Shows subtle "Hidden by Kiro" badges

**Auto-Skip**
- Automatically skips blocked tracks during playback
- Shows toast notification with skip reason
- Configurable skip delay (0-3 seconds)

**Override Controls**
- Right-click any hidden content for options:
  - **Play Once**: Temporarily allow this track
  - **Remove from DNP**: Remove artist from blocklist
  - **Add to DNP**: Add artist to blocklist

### Installation

1. Install the browser extension from:
   - [Chrome Web Store](https://chrome.google.com/webstore)
   - [Firefox Add-ons](https://addons.mozilla.org)
   - [Edge Add-ons](https://microsoftedge.microsoft.com/addons)

2. Pin the extension to your toolbar
3. Click the extension icon and log in
4. The extension will automatically sync with your DNP list

## 📊 Enforcement Planning

### Dry Run Preview

Before making changes, always run a dry run to see the impact:

1. Go to **Enforcement** → **Plan Enforcement**
2. Select **Spotify** as the target platform
3. Choose your enforcement options
4. Click **Generate Preview**

The preview shows:
```
📊 Enforcement Impact Preview

Liked Songs: 45 tracks to remove
├── Primary artists: 32 tracks
├── Featured artists: 8 tracks
└── Collaborations: 5 tracks

Saved Albums: 12 albums to remove
├── Full albums: 10 albums
└── Compilation albums: 2 albums

Playlists: 8 playlists to modify
├── Personal playlists: 6 playlists (67 tracks)
├── Collaborative playlists: 2 playlists (12 tracks)
└── Warnings: 2 restricted operations

Following: 3 artists to unfollow

Estimated time: 2 minutes 30 seconds
```

### Execution

1. Review the preview carefully
2. Adjust settings if needed
3. Click **Execute Enforcement**
4. Monitor progress in real-time
5. Review the completion report

## 🔄 Rollback & Undo

### What Can Be Rolled Back

✅ **Reversible Actions**
- Re-add liked songs (if still available)
- Re-save albums to library
- Re-add tracks to playlists (with position)
- Re-follow artists

❌ **Irreversible Actions**
- Tracks removed from collaborative playlists by others
- Deleted playlists (rare edge case)
- Changes to Spotify's recommendation algorithm

### How to Rollback

1. Go to **History** → **Enforcement History**
2. Find the execution you want to rollback
3. Click **Rollback** (available for 7 days)
4. Choose rollback type:
   - **Full**: Undo all changes
   - **Partial**: Select specific platforms
   - **Selective**: Choose individual actions

## ⚡ Performance & Rate Limits

### Spotify API Limits

- **Rate Limit**: 100 requests per minute per user
- **Batch Size**: Up to 50 items per request
- **Concurrent Operations**: 5 simultaneous requests

### Optimization Features

**Smart Batching**
- Groups operations by playlist for efficiency
- Uses delta removal to minimize API calls
- Processes items in optimal order

**Rate Limit Handling**
- Automatically respects Spotify's rate limits
- Shows estimated completion times
- Queues operations when limits are reached

**Resume Capability**
- Can resume interrupted operations
- Saves progress checkpoints
- Handles network interruptions gracefully

## 🔍 Troubleshooting

### Common Issues

**Connection Problems**
```
❌ "Spotify connection expired"
```
**Solution**: Go to Settings → Connected Services → Reconnect Spotify

**Permission Errors**
```
❌ "Insufficient permissions to modify playlist"
```
**Solution**: Check if playlist is collaborative or owned by another user

**Rate Limit Warnings**
```
⚠️ "Rate limit reached, operation queued"
```
**Solution**: Wait for the queue to process, or try during off-peak hours

### Advanced Troubleshooting

**Missing Tracks After Enforcement**
1. Check if tracks were actually removed or just hidden
2. Verify the artist wasn't added to DNP list accidentally
3. Look for the track in "Recently Played" to confirm removal

**Playlist Order Changed**
1. This can happen with large playlists due to API limitations
2. Use "Conservative" mode to minimize reordering
3. Consider processing smaller playlists separately

**Featured Artists Not Detected**
1. Ensure "Block featuring artists" is enabled
2. Some tracks may not have proper metadata
3. Report missing detections to improve the algorithm

## 📈 Best Practices

### Before Enforcement

1. **Backup Important Playlists**: Export playlists you care about
2. **Start Small**: Test with a few artists before bulk operations
3. **Review Community Lists**: Check subscribed lists for unexpected additions
4. **Check Collaborative Playlists**: Warn collaborators about upcoming changes

### During Enforcement

1. **Monitor Progress**: Stay on the page during execution
2. **Don't Close Browser**: Interruptions can cause incomplete operations
3. **Check Warnings**: Address any permission or access issues

### After Enforcement

1. **Review Results**: Check the completion report for errors
2. **Test Playback**: Verify auto-skip is working as expected
3. **Adjust Settings**: Fine-tune based on results
4. **Schedule Regular Enforcement**: Set reminders for periodic cleanup

## 🎯 Pro Tips

### Maximizing Effectiveness

**Use Tags for Organization**
```
Personal preferences: #personal
Explicit content: #explicit
Temporary blocks: #temp
Community list items: #community
```

**Optimize for Your Listening Habits**
- If you mostly use playlists: Focus on playlist enforcement
- If you use radio/discover: Enable recommendation filtering
- If you share playlists: Use conservative mode for collaborative lists

**Timing Your Enforcement**
- Run enforcement during off-peak hours (early morning)
- Avoid running during Spotify maintenance windows
- Consider time zones if you have international collaborative playlists

### Advanced Workflows

**Seasonal Cleanup**
1. Export your current library state
2. Run comprehensive enforcement
3. Review and adjust based on listening patterns
4. Update community list subscriptions

**Collaborative Playlist Management**
1. Create a "staging" playlist for testing
2. Use conservative mode for shared playlists
3. Communicate changes to collaborators
4. Keep rollback available for 7 days

---

## Need Help?

- **Spotify-specific issues**: Check [Spotify's API status](https://developer.spotify.com/documentation/web-api/)
- **Feature requests**: Visit our [community forum](https://community.nodrakeinthe.house)
- **Bug reports**: Email [support@nodrakeinthe.house](mailto:support@nodrakeinthe.house)