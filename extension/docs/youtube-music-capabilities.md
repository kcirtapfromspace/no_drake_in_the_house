# YouTube Music Capabilities and Limitations

## Overview

Kiro's YouTube Music integration operates in **preview-only mode** due to YouTube Music's Terms of Service and API limitations. This document explains what Kiro can and cannot do on YouTube Music, and provides best practices for effective content blocking.

## What Kiro CAN Do ✅

### Visual Content Filtering
- **Hide blocked artists** with visual overlays and grayed-out styling
- **Add blocked indicators** showing "🚫 Blocked" badges on content
- **Provide override controls** with "Play Once" and "Unblock" options
- **Filter search results** by hiding blocked artist content

### Auto-Skip Functionality
- **Automatically skip blocked tracks** during playback
- **Show skip notifications** with artist name and reason
- **Prevent infinite loops** with smart skip attempt limiting
- **ToS-compliant skipping** using simulated user interactions

### Recommendation Filtering
- **Hide blocked artists** from home page recommendations
- **Filter mix suggestions** and related content sections
- **Block radio seeds** to prevent blocked artists in generated stations
- **Queue management** by removing blocked tracks from autoplay

### Data Export/Import
- **Export blocked content** found on current page to JSON
- **Import blocklist data** from previous exports or other devices
- **Manual sync workflows** for library cleanup
- **Export history tracking** for audit purposes

### "Not Interested" Automation
- **Mark blocked content** as "Not Interested" automatically
- **Train YouTube's algorithm** to reduce blocked artist recommendations
- **Respectful rate limiting** to avoid triggering anti-automation measures

## What Kiro CANNOT Do ❌

### API Limitations
- **Remove tracks from library** - YouTube Music API doesn't support bulk library modifications
- **Modify playlists automatically** - Playlist editing requires manual user action
- **Block artists at account level** - No API endpoint for permanent artist blocking
- **Access private user data** - Cannot read full library contents without explicit user action

### Terms of Service Restrictions
- **Bypass recommendation algorithms** - Cannot completely override YouTube's content delivery
- **Automated bulk actions** - Must respect rate limits and user interaction patterns
- **Direct database modifications** - Cannot alter YouTube's internal data structures

## Best Practices 💡

### Regular Maintenance
1. **Weekly exports** - Export blocked content weekly to track what needs manual removal
2. **Manual cleanup** - Remove exported items from your YouTube Music library manually
3. **Playlist reviews** - Periodically check playlists for blocked content
4. **Library audits** - Use export data to identify patterns in blocked content

### Algorithm Training
1. **Use "Not Interested"** - Let Kiro mark blocked content as not interested
2. **Avoid blocked artists** - Don't manually play blocked artists even temporarily
3. **Curate radio seeds** - Choose non-blocked artists when creating radio stations
4. **Thumbs down** - Use YouTube Music's thumbs down on blocked content

### Workflow Optimization
1. **Start with export** - Always export first to see what content is blocked
2. **Batch manual actions** - Group similar manual actions together for efficiency
3. **Import after cleanup** - Re-import after manual library cleanup to update blocklist
4. **Monitor recommendations** - Watch for blocked artists in recommendations and mark as not interested

## Technical Implementation

### Artist Detection Strategies
- **Multiple selectors** - Uses various CSS selectors and data attributes
- **Context awareness** - Different detection logic for different page types
- **Fallback methods** - Multiple extraction strategies for robust detection
- **Performance optimization** - Bloom filter for O(1) artist lookup

### ToS Compliance
- **Simulated user interactions** - All actions appear as natural user behavior
- **Rate limiting** - Respects YouTube's rate limits and usage patterns
- **No API abuse** - Doesn't use undocumented or private APIs
- **Privacy focused** - Minimal data collection, local processing

### Error Handling
- **Graceful degradation** - Continues working even if some features fail
- **Retry logic** - Automatically retries failed operations with backoff
- **User feedback** - Clear notifications about what worked and what didn't
- **Logging** - Comprehensive logging for troubleshooting

## Troubleshooting

### Common Issues

**Blocked content still appears**
- Refresh the page after updating your blocklist
- Check if the artist name matches exactly in your DNP list
- Some content may load before Kiro can process it

**Auto-skip not working**
- Ensure the next button is visible and enabled
- Check browser console for any JavaScript errors
- Try refreshing the page to reinitialize the extension

**Export contains no data**
- Make sure you're on a page with music content (home, search, playlists)
- Verify your DNP list has artists added
- Check that the extension has permission to run on YouTube Music

**Import fails**
- Ensure the JSON file is valid and from a Kiro export
- Check that the file contains the expected data structure
- Try importing a smaller file to test the functionality

### Getting Help

1. **Check browser console** - Look for error messages in developer tools
2. **Review export data** - Verify your export files contain expected content
3. **Test on different pages** - Try the extension on various YouTube Music pages
4. **Update extension** - Ensure you're running the latest version

## Future Enhancements

### Planned Features
- **Enhanced detection** - Improved artist detection algorithms
- **Better automation** - More sophisticated "Not Interested" automation
- **Playlist analysis** - Tools to analyze playlists for blocked content
- **Statistics dashboard** - Detailed stats on blocked content and effectiveness

### API Wishlist
- **Official blocking API** - YouTube Music API for permanent artist blocking
- **Bulk library operations** - API endpoints for bulk library modifications
- **Recommendation control** - API to influence recommendation algorithms
- **Playlist management** - Programmatic playlist editing capabilities

---

*This document is updated regularly as YouTube Music's features and APIs evolve. Last updated: December 2024*