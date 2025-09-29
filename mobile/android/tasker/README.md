# Tasker Integration for DNP Manager

This directory contains Tasker profiles and tasks for automating DNP Manager functionality.

## Available Profiles

### DNP_Add_Artist.prf.xml
- **Trigger**: Voice command or custom event
- **Action**: Adds an artist to your DNP list
- **Usage**: "Hey Google, trigger add artist to DNP"
- **Variables Set**: 
  - `%success` - true/false
  - `%artist_name` - name of added artist
  - `%error` - error message if failed

### DNP_Status_Check.prf.xml
- **Trigger**: Voice command or custom event
- **Action**: Checks and speaks your current DNP status
- **Usage**: "Hey Google, trigger check DNP status"
- **Variables Set**:
  - `%success` - true/false
  - `%dnp_count` - number of blocked artists
  - `%connected_services` - array of connected services
  - `%last_enforcement` - when enforcement last ran
  - `%error` - error message if failed

## Installation Instructions

1. **Install Tasker** from Google Play Store
2. **Import Profiles**:
   - Open Tasker
   - Long press on the Profiles tab
   - Select "Import"
   - Choose the `.prf.xml` files from this directory
3. **Configure API Access**:
   - Set variable `%dnp_api_token` with your API token
   - Set variable `%dnp_api_base_url` (optional, defaults to https://api.nodrakeinthe.house)
4. **Enable Profiles**:
   - Tap each imported profile to enable it
   - Grant any requested permissions

## Custom Intents

You can also trigger DNP actions directly using Android intents:

### Add Artist
```
Intent: house.nodrakeinthe.TASKER_ADD_ARTIST
Extras:
  - artist_name (String): Name of artist to add
  - tags (String[]): Optional tags (default: ["tasker-added"])
  - note (String): Optional note (default: "Added via Tasker")
```

### Remove Artist
```
Intent: house.nodrakeinthe.TASKER_REMOVE_ARTIST
Extras:
  - artist_name (String): Name of artist to remove
  OR
  - artist_id (String): ID of artist to remove
```

### Check Status
```
Intent: house.nodrakeinthe.TASKER_CHECK_STATUS
No extras required
```

## Voice Commands Setup

To use with Google Assistant:

1. **Create Assistant Routines**:
   - Open Google Assistant settings
   - Go to "Routines"
   - Create new routine with phrase like "Add artist to DNP"
   - Add action: "Send broadcast intent"
   - Set intent to trigger Tasker profile

2. **Example Phrases**:
   - "Add [artist name] to my DNP list"
   - "Check my DNP status"
   - "Remove [artist name] from DNP"
   - "How many artists are blocked?"

## Advanced Automation Examples

### Auto-Skip Based on Time
```xml
<!-- Profile: Skip aggressive artists during work hours -->
<Profile>
  <Time>09:00-17:00</Time>
  <Application>com.spotify.music</Application>
  <Task>
    <!-- Check if current artist is in "aggressive" tag -->
    <!-- Skip if found -->
  </Task>
</Profile>
```

### Location-Based DNP
```xml
<!-- Profile: Different DNP rules at home vs work -->
<Profile>
  <Location>Home</Location>
  <Task>
    <!-- Load "home" DNP list -->
  </Task>
</Profile>
```

### Automatic Enforcement
```xml
<!-- Profile: Run enforcement daily at 2 AM -->
<Profile>
  <Time>02:00</Time>
  <Task>
    <!-- Trigger enforcement via API -->
  </Task>
</Profile>
```

## Troubleshooting

### Common Issues

1. **"Permission Denied" errors**:
   - Ensure DNP Manager app has notification access
   - Grant Tasker accessibility permissions
   - Check that API token is valid

2. **Voice commands not working**:
   - Verify Google Assistant integration is set up
   - Check that Tasker profiles are enabled
   - Test with manual profile execution first

3. **API calls failing**:
   - Verify internet connection
   - Check API token expiration
   - Confirm base URL is correct

### Debug Mode

Enable debug mode in Tasker to see detailed logs:
1. Tasker → Preferences → UI → Beginner Mode (disable)
2. Tasker → Preferences → Misc → Enable "Run Log"
3. Check logs for detailed error information

## Variables Reference

All Tasker variables set by DNP Manager:

| Variable | Type | Description |
|----------|------|-------------|
| `%success` | Boolean | Whether the operation succeeded |
| `%error` | String | Error message if operation failed |
| `%artist_name` | String | Name of the artist |
| `%artist_id` | String | Unique ID of the artist |
| `%dnp_count` | Integer | Number of artists in DNP list |
| `%connected_services` | Array | List of connected streaming services |
| `%last_enforcement` | String | Timestamp of last enforcement |