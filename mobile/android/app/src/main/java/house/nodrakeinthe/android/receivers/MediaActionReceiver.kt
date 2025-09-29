package house.nodrakeinthe.android.receivers

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import house.nodrakeinthe.android.api.DNPApiClient
import house.nodrakeinthe.android.utils.BloomFilterManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class MediaActionReceiver : BroadcastReceiver() {
    
    companion object {
        private const val TAG = "MediaActionReceiver"
        
        const val ACTION_SKIP_CONFIRMED = "house.nodrakeinthe.SKIP_CONFIRMED"
        const val ACTION_UNDO_SKIP = "house.nodrakeinthe.UNDO_SKIP"
        const val ACTION_REMOVE_FROM_DNP = "house.nodrakeinthe.REMOVE_FROM_DNP"
        const val ACTION_ADD_TO_DNP = "house.nodrakeinthe.ADD_TO_DNP"
    }
    
    override fun onReceive(context: Context, intent: Intent) {
        Log.d(TAG, "Received media action: ${intent.action}")
        
        val pendingResult = goAsync()
        val scope = CoroutineScope(Dispatchers.IO)
        
        scope.launch {
            try {
                when (intent.action) {
                    ACTION_SKIP_CONFIRMED -> handleSkipConfirmed(context, intent)
                    ACTION_UNDO_SKIP -> handleUndoSkip(context, intent)
                    ACTION_REMOVE_FROM_DNP -> handleRemoveFromDNP(context, intent)
                    ACTION_ADD_TO_DNP -> handleAddToDNP(context, intent)
                    else -> Log.w(TAG, "Unknown action: ${intent.action}")
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error handling media action", e)
            } finally {
                pendingResult.finish()
            }
        }
    }
    
    private suspend fun handleSkipConfirmed(context: Context, intent: Intent) {
        val artist = intent.getStringExtra("artist") ?: return
        val title = intent.getStringExtra("title") ?: return
        
        Log.i(TAG, "Skip confirmed for: $title by $artist")
        // This action is just for logging/analytics
        // The actual skip was already performed by the MediaNotificationListener
    }
    
    private suspend fun handleUndoSkip(context: Context, intent: Intent) {
        val artist = intent.getStringExtra("artist") ?: return
        val title = intent.getStringExtra("title") ?: return
        
        Log.i(TAG, "Undo skip requested for: $title by $artist")
        
        // In a real implementation, this would attempt to go back to the previous track
        // For now, we'll just show a message that undo isn't possible
        // since most streaming services don't support going back to a specific track
        
        // Could potentially add the track to a "recently skipped" list for manual replay
    }
    
    private suspend fun handleRemoveFromDNP(context: Context, intent: Intent) {
        val artist = intent.getStringExtra("artist") ?: return
        
        try {
            val apiClient = DNPApiClient(context)
            val bloomFilterManager = BloomFilterManager(context)
            
            // Get DNP list to find the artist ID
            val dnpList = apiClient.getDNPList()
            val matchingArtist = dnpList.artists.find { 
                it.name.equals(artist, ignoreCase = true) 
            }
            
            if (matchingArtist != null) {
                apiClient.removeArtistFromDNP(matchingArtist.id)
                
                // Refresh bloom filter
                bloomFilterManager.refreshFilter()
                
                Log.i(TAG, "Removed artist from DNP: $artist")
                
                // Could show a success notification here
            } else {
                Log.w(TAG, "Artist not found in DNP list: $artist")
            }
            
        } catch (e: Exception) {
            Log.e(TAG, "Failed to remove artist from DNP", e)
        }
    }
    
    private suspend fun handleAddToDNP(context: Context, intent: Intent) {
        val artist = intent.getStringExtra("artist") ?: return
        
        try {
            val apiClient = DNPApiClient(context)
            val bloomFilterManager = BloomFilterManager(context)
            
            // Search for the artist first
            val searchResults = apiClient.searchArtists(artist, limit = 1)
            if (searchResults.artists.isNotEmpty()) {
                val foundArtist = searchResults.artists.first()
                
                apiClient.addArtistToDNP(
                    foundArtist.id,
                    listOf("notification-added"),
                    "Added via notification action"
                )
                
                // Update bloom filter
                bloomFilterManager.addArtist(foundArtist.name)
                
                Log.i(TAG, "Added artist to DNP: $artist")
                
                // Could show a success notification here
            } else {
                Log.w(TAG, "Artist not found in search: $artist")
            }
            
        } catch (e: Exception) {
            Log.e(TAG, "Failed to add artist to DNP", e)
        }
    }
}