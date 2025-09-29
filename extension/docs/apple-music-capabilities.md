# Apple Music Integration Capabilities

## Overview

The Apple Music integration provides limited functionality due to Apple Music's restricted web API access. This document outlines what is supported, what has limitations, and what requires manual user action.

## Capability Matrix

| Feature | Support Level | Description |
|---------|---------------|-------------|
| **Visual Content Blocking** | ✅ Full Support | Hide blocked artists in web interface |
| **Artist Detection** | ✅ Full Support | Detect artists from various page elements |
| **DNP List Management** | ✅ Full Support | Add/remove artists from blocklist |
| **Auto-Skip Tracks** | ❌ Not Supported | Apple Music doesn't allow programmatic playback control |
| **Library Modification** | ❌ Not Supported | Cannot automatically remove tracks/albums |
| **Playlist Modification** | ❌ Not Supported | Cannot automatically modify playlists |
| **Now Playing Detection** | ⚠️ Limited | Can detect but cannot auto-skip |
| **Search Filtering** | ✅ Full Support | Hide blocked artists in search results |
| **Recommendation Filtering** | ⚠️ Limited | Visual hiding only, cannot modify algorithm |

## Supported Operations

### ✅ Fully Supported

1. **Visual Content Blocking**
   - Hide artist cards and links
   - Dim blocked tracks in playlists
   - Show "Blocked by Kiro" indicators
   - Overlay blocked artist pages

2. **Artist Detection**
   - Extract artist info from links (`/artist/` URLs)
   - Detect artists in track listings
   - Identify artists in now-playing area
   - Parse artist information from search results

3. **User Interface Integration**
   - Quick action menus for unblocking
   - Notification system for user feedback
   - Settings integration with main extension

### ⚠️ Limited Support

1. **Now Playing Detection**
   - **What works**: Detect when blocked artist is playing
   - **Limitation**: Cannot automatically skip track
   - **User action**: Manual skip required

2. **Library Analysis**
   - **What works**: Scan and identify blocked content
   - **Limitation**: Cannot automatically remove content
   - **User action**: Manual removal required via Apple Music app

3. **Playlist Management**
   - **What works**: Identify blocked tracks in playlists
   - **Limitation**: Cannot modify playlist contents
   - **User action**: Manual track removal required

### ❌ Not Supported

1. **Automatic Playback Control**
   - Cannot pause/skip tracks programmatically
   - Cannot modify playback queue
   - Cannot control shuffle/repeat settings

2. **Library Modifications**
   - Cannot remove tracks from library
   - Cannot unlike/unfavorite content
   - Cannot modify user's saved content

3. **Playlist Modifications**
   - Cannot add/remove tracks from playlists
   - Cannot create/delete playlists
   - Cannot modify playlist metadata

## Technical Implementation

### Artist Detection Strategies

1. **URL-based Detection** (Primary)
   ```javascript
   // Extract from Apple Music artist URLs
   if (element.href.includes('/artist/')) {
     const artistId = element.href.split('/artist/')[1]?.split('/')[0];
     // ...
   }
   ```

2. **DOM Structure Analysis**
   ```javascript
   // Track rows in song lists
   '.songs-list-row, .tracklist__item'
   
   // Artist cards in browse/search
   '.grid-item--artist, .artist-lockup'
   
   // Now playing controls
   '.web-chrome-playback-controls, .playback-controls'
   ```

3. **Text Content Parsing**
   ```javascript
   // Fallback for elements without direct links
   const artistText = element.querySelector('.songs-list-row__by-line');
   ```

### Visual Blocking Implementation

1. **Content Hiding**
   ```css
   .kiro-hidden {
     opacity: 0.3 !important;
     filter: grayscale(100%) !important;
     pointer-events: none !important;
   }
   ```

2. **Overlay System**
   - Shadow DOM isolation for UI components
   - Positioned overlays with user controls
   - Apple Music-styled interface elements

3. **Notification System**
   - Native-looking notifications
   - Action buttons for user interaction
   - Automatic dismissal with manual override

## User Experience

### What Users See

1. **Blocked Content Indicators**
   - Grayed-out artist cards and tracks
   - "🚫 Blocked" badges on content
   - Full-page overlays for artist pages

2. **Quick Action Menus**
   - "Show Once" - temporarily reveal content
   - "Unblock Artist" - remove from DNP list
   - "Block Permanently" - add to DNP list

3. **Informational Messages**
   - Capability limitations explained
   - Manual action instructions provided
   - Clear feedback on user actions

### Manual Actions Required

1. **Track Skipping**
   - User must manually click next/skip button
   - Extension shows notification when blocked track detected
   - No automatic playback control available

2. **Library Cleanup**
   - User must open Apple Music app
   - Manually remove blocked tracks/albums
   - Extension provides export of blocked content list

3. **Playlist Management**
   - User must edit playlists manually
   - Extension identifies blocked tracks
   - Manual removal required for each track

## Comparison with Other Platforms

| Feature | Spotify | Apple Music | YouTube Music |
|---------|---------|-------------|---------------|
| Auto-skip | ✅ Yes | ❌ No | ⚠️ Limited |
| Library modification | ✅ Yes | ❌ No | ❌ No |
| Playlist modification | ✅ Yes | ❌ No | ❌ No |
| Visual blocking | ✅ Yes | ✅ Yes | ✅ Yes |
| API access | ✅ Full | ❌ Limited | ❌ Very Limited |

## Future Improvements

### Potential Enhancements

1. **Enhanced Detection**
   - Machine learning for better artist recognition
   - Featured artist detection in track titles
   - Collaboration detection from metadata

2. **Better User Experience**
   - Keyboard shortcuts for quick actions
   - Bulk operations for multiple tracks
   - Integration with Apple Music app (if API access improves)

3. **Export/Import Features**
   - Export blocked content lists
   - Import from other platforms
   - Sync with main Kiro service

### API Limitations

Apple Music's web API restrictions mean that many features available on other platforms cannot be implemented. These limitations are imposed by Apple and cannot be circumvented through the browser extension.

**Key Restrictions:**
- No programmatic playback control
- No library modification capabilities
- No playlist editing permissions
- Limited metadata access
- No recommendation algorithm access

## Troubleshooting

### Common Issues

1. **Artist Not Detected**
   - Check if artist page has proper URL structure
   - Verify artist name spelling in DNP list
   - Try refreshing the page

2. **Visual Blocking Not Working**
   - Ensure extension has proper permissions
   - Check if Apple Music updated their DOM structure
   - Verify content script is loaded

3. **Quick Actions Not Responding**
   - Check browser console for errors
   - Verify connection to Kiro backend
   - Try reloading the page

### Reporting Issues

When reporting issues with Apple Music integration:

1. Include the specific Apple Music page URL
2. Describe the expected vs actual behavior
3. Check browser console for error messages
4. Note if the issue occurs on other platforms

## Conclusion

While Apple Music's API limitations prevent full automation like other platforms, the visual blocking and detection capabilities provide significant value to users who want to avoid specific artists. The extension clearly communicates these limitations and provides the best possible experience within Apple's constraints.

Users should understand that manual action is required for most content management tasks, but the extension makes it easy to identify what needs to be removed and provides clear instructions for doing so.