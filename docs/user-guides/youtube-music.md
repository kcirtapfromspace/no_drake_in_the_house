# YouTube Music Integration Guide

YouTube Music integration is web-based due to limited API access, focusing on browser extension functionality for content filtering and auto-skip capabilities.

## 🎯 What You Can Do

### ✅ Supported Features (Web Extension)

- **Content Filtering**: Hide blocked artists and tracks in the web interface
- **Auto-Skip**: Automatically skip blocked tracks during playback
- **Visual Indicators**: Show "Hidden by Kiro" badges on filtered content
- **Override Controls**: Temporarily allow blocked content with right-click options
- **Recommendation Filtering**: Hide blocked artists from recommendation sections

### ⚠️ Limited Features

- **Manual Synchronization**: No direct API access for library modifications
- **Web-Only**: Only works in the YouTube Music web player (music.youtube.com)
- **Export/Import Workflows**: Manual processes for library management
- **Preview Mode**: Shows what would be changed without direct modification

### ❌ Unsupported Features

- **Direct Library Modification**: Cannot automatically remove songs from library
- **Playlist Management**: Cannot directly modify playlists via API
- **Mobile App Integration**: Extension only works in web browsers
- **Offline Content**: Cannot filter downloaded/offline content

## 🔗 Setting Up YouTube Music Integration

### Step 1: Install Browser Extension

1. Install the browser extension from:
   - [Chrome Web Store](https://chrome.google.com/webstore)
   - [Firefox Add-ons](https://addons.mozilla.org)
   - [Edge Add-ons](https://microsoftedge.microsoft.com/addons)

2. Pin the extension to your browser toolbar
3. Navigate to [music.youtube.com](https://music.youtube.com)
4. Log in to your YouTube Music account

### Step 2: Configure Extension

1. Click the extension icon in your browser
2. Log in to your Kiro account
3. The extension will automatically sync your DNP list
4. Configure filtering preferences:

```
Content Filtering Options:
☑️ Hide artist tiles in browse sections
☑️ Hide tracks in playlists and albums
☑️ Hide recommendations and suggestions
☑️ Show "Hidden by Kiro" badges
☑️ Enable auto-skip functionality
```

### Step 3: Verify Setup

After configuration, you should see:
- Blocked content is visually hidden or dimmed
- "Hidden by Kiro" badges appear on filtered items
- Auto-skip works during playback
- Right-click options are available on hidden content

## 🌐 Browser Extension Features

### Content Filtering

**Visual Hiding**
- Artist tiles are hidden in browse sections
- Track rows are dimmed in playlists and albums
- Recommendation cards are filtered out
- Search results show filtered items with badges

**Smart Detection**
The extension uses multiple strategies to identify content:
- **Data attributes**: YouTube's internal track/artist IDs
- **ARIA labels**: Accessibility labels for screen readers
- **Text content**: Artist and track names in the interface
- **URL patterns**: YouTube Music URL structures

### Auto-Skip Functionality

**Playback Control**
- Automatically skips blocked tracks during playback
- Configurable skip delay (0-5 seconds)
- Shows toast notification with skip reason
- Handles both single tracks and continuous playback

**Skip Behavior**
```
Skip Modes:
• Immediate: Skip as soon as track starts (0s delay)
• Quick: Skip after 1 second (allows for manual override)
• Delayed: Skip after 3-5 seconds (more time to override)
```

### Override Controls

Right-click any hidden or skipped content for options:
- **Play Once**: Allow this track to play temporarily
- **Remove from DNP**: Remove artist from your blocklist
- **Add to DNP**: Add artist to your blocklist
- **Report Issue**: Report detection problems

## 📊 YouTube Music Preview Mode

Since direct API access is limited, the platform provides preview functionality:

### Library Analysis

1. Go to **Enforcement** → **YouTube Music Preview**
2. Click **Analyze Library** (requires manual export)
3. Upload your YouTube Music data export
4. View impact analysis:

```
📊 YouTube Music Library Analysis

Based on your exported data:

Library Songs: 1,247 tracks analyzed
├── Blocked tracks found: 89 tracks
├── Featured appearances: 23 tracks
├── Collaborations: 12 tracks
└── Recommendations: Manual removal required

Playlists: 15 playlists analyzed
├── Personal playlists: 12 playlists (67 blocked tracks)
├── Collaborative playlists: 3 playlists (15 blocked tracks)
└── Auto-generated playlists: Cannot be modified

⚠️ Manual Actions Required:
- Remove 89 tracks from library manually
- Clean 12 personal playlists
- Use extension for ongoing filtering
```

### Export/Import Workflow

**Step 1: Export Your Data**
1. Go to [Google Takeout](https://takeout.google.com)
2. Select "YouTube and YouTube Music"
3. Choose "YouTube Music" data
4. Download your archive

**Step 2: Upload for Analysis**
1. In Kiro, go to **YouTube Music** → **Import Data**
2. Upload your YouTube Music data export
3. Wait for analysis to complete

**Step 3: Review Recommendations**
1. View the list of tracks/artists to remove
2. Export the removal list as CSV
3. Use the list to manually clean your library

## 🔧 Configuration Options

### Extension Settings

**Filtering Aggressiveness**
```
Conservative:
☑️ Hide primary artists only
☐ Hide featured artists
☐ Hide collaborations
☑️ Show override controls

Moderate:
☑️ Hide primary artists
☑️ Hide featured artists
☑️ Hide collaborations
☑️ Show override controls

Aggressive:
☑️ Hide all appearances
☑️ Hide similar artists
☑️ Hide recommendations
☑️ Auto-skip immediately
```

**Visual Preferences**
```
Display Options:
☑️ Show "Hidden by Kiro" badges
☑️ Dim hidden content (vs complete hiding)
☑️ Show skip notifications
☑️ Enable right-click context menu
```

### Auto-Skip Settings

**Skip Timing**
- **Immediate**: 0 seconds (skip as soon as detected)
- **Quick**: 1 second (brief delay for manual override)
- **Standard**: 3 seconds (comfortable override window)
- **Delayed**: 5 seconds (maximum override time)

**Skip Notifications**
```
Notification Options:
☑️ Show toast notifications
☑️ Include artist name in notification
☑️ Show skip reason
☐ Play notification sound
```

## 🔍 Manual Library Management

### Cleaning Your Library

Since automatic library modification isn't available, use these manual workflows:

**Step 1: Generate Removal List**
1. Use the preview mode to analyze your library
2. Export the list of tracks to remove
3. Sort by playlist or album for easier manual removal

**Step 2: Manual Removal Process**
1. Open YouTube Music in a separate tab
2. Use the removal list to find and remove tracks
3. The extension will help by highlighting blocked content
4. Check off completed removals in the list

**Step 3: Ongoing Maintenance**
1. Use the extension to prevent adding new blocked content
2. Regularly review your library for new additions
3. Update your DNP list as needed

### Playlist Management

**Personal Playlists**
1. Open each playlist in YouTube Music
2. Use the extension to identify blocked tracks (they'll be dimmed)
3. Manually remove the highlighted tracks
4. The extension will update in real-time as you remove items

**Collaborative Playlists**
1. Extension will show blocked content but cannot remove it
2. Consider creating personal copies of important collaborative playlists
3. Use the extension for visual filtering only

## 🔄 Synchronization

### Keeping Data in Sync

**DNP List Updates**
- Extension automatically syncs with your Kiro account
- Changes to your DNP list appear in the extension within minutes
- Manual refresh available via extension popup

**Library Changes**
- Manual library changes are not automatically tracked
- Use periodic analysis to check for new blocked content
- Export/import workflow helps maintain accuracy

## 🔍 Troubleshooting

### Common Issues

**Extension Not Working**
```
❌ "Content not being filtered"
```
**Solutions**:
1. Refresh the YouTube Music page
2. Check if you're logged in to both YouTube Music and Kiro
3. Verify extension permissions
4. Clear browser cache for music.youtube.com

**Auto-Skip Not Working**
```
❌ "Blocked tracks still playing"
```
**Solutions**:
1. Check auto-skip is enabled in extension settings
2. Verify the track is actually in your DNP list
3. Try increasing the skip delay
4. Check browser console for error messages

**Missing Content Detection**
```
⚠️ "Some blocked artists not being detected"
```
**Solutions**:
1. Report the issue via right-click menu
2. Check if artist names match exactly
3. Try refreshing the page
4. Update the extension to the latest version

### Advanced Troubleshooting

**Performance Issues**
1. Extension may slow down on very large playlists (1000+ tracks)
2. Consider using "Conservative" filtering mode
3. Disable visual effects if experiencing lag
4. Close other browser tabs to free up memory

**Detection Accuracy**
1. YouTube Music's interface changes frequently
2. Extension updates may be needed for new layouts
3. Report detection issues to help improve accuracy
4. Use manual override controls when needed

## 📈 Best Practices

### Effective YouTube Music Usage

**Browser Setup**
1. Use a dedicated browser profile for music
2. Keep the extension updated
3. Pin YouTube Music tab for easy access
4. Enable notifications for skip alerts

**Library Maintenance**
1. Run periodic library analysis (monthly)
2. Keep your DNP list updated
3. Use tags to organize blocked content
4. Export your library regularly as backup

**Playlist Management**
1. Create personal copies of important collaborative playlists
2. Use descriptive playlist names for easier management
3. Regularly review auto-generated playlists
4. Consider using YouTube Music's "Don't play this artist" feature alongside Kiro

### Workflow Optimization

**Daily Usage**
1. Let the extension handle filtering automatically
2. Use override controls when needed
3. Add new artists to DNP list via right-click
4. Monitor skip notifications for accuracy

**Weekly Maintenance**
1. Review skip logs for false positives
2. Update extension settings based on usage
3. Check for new blocked content in library
4. Sync DNP list changes across devices

**Monthly Cleanup**
1. Export and analyze library data
2. Manually remove accumulated blocked content
3. Review and update collaborative playlists
4. Check extension performance and settings

## 🎯 Pro Tips

### Maximizing YouTube Music Integration

**Use Multiple Strategies**
1. Combine extension filtering with manual curation
2. Use YouTube Music's built-in "Don't play this artist" feature
3. Create "clean" playlists without blocked content
4. Use the extension's preview mode for planning

**Cross-Platform Coordination**
1. Keep your DNP list consistent across platforms
2. Use tags to identify YouTube Music specific blocks
3. Export data regularly for backup
4. Coordinate with other streaming service enforcement

**Advanced Filtering**
1. Use community lists for broader filtering
2. Create separate DNP lists for different contexts
3. Use the extension's reporting feature to improve detection
4. Participate in beta testing for new features

---

## YouTube Music Limitations

### Platform Constraints

**API Limitations**
- No official API for library modification
- Limited access to user data
- Cannot modify mobile app behavior
- Dependent on web interface stability

**Technical Constraints**
- Browser extension only works in web browsers
- Cannot filter offline/downloaded content
- Limited to visual filtering and auto-skip
- Dependent on YouTube Music's interface structure

### Future Improvements

**Planned Features**
- Enhanced detection algorithms
- Better performance optimization
- Mobile app integration (if APIs become available)
- Improved synchronization with other platforms

**Community Contributions**
- Report detection issues to improve accuracy
- Suggest new filtering strategies
- Participate in beta testing
- Share workflow improvements

---

## Need Help?

- **Extension Issues**: Check browser console for error messages
- **YouTube Music Problems**: Visit [YouTube Music Help](https://support.google.com/youtubemusic)
- **Detection Issues**: Use the extension's "Report Issue" feature
- **General Support**: Email [support@nodrakeinthe.house](mailto:support@nodrakeinthe.house)
- **Community**: Join our [forum](https://community.nodrakeinthe.house) for tips and tricks