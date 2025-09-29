package house.nodrakeinthe.android.receivers

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.util.Log
import house.nodrakeinthe.android.api.DNPApiClient
import house.nodrakeinthe.android.utils.TaskerResultHelper
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class TaskerReceiver : BroadcastReceiver() {
    
    private val apiClient = DNPApiClient()
    
    companion object {
        private const val TAG = "TaskerReceiver"
        
        // Tasker action constants
        const val ACTION_ADD_ARTIST = "house.nodrakeinthe.TASKER_ADD_ARTIST"
        const val ACTION_REMOVE_ARTIST = "house.nodrakeinthe.TASKER_REMOVE_ARTIST"
        const val ACTION_CHECK_STATUS = "house.nodrakeinthe.TASKER_CHECK_STATUS"
        const val ACTION_SKIP_TRACK = "house.nodrakeinthe.TASKER_SKIP_TRACK"
        
        // Extra keys
        const val EXTRA_ARTIST_NAME = "artist_name"
        const val EXTRA_ARTIST_ID = "artist_id"
        const val EXTRA_TAGS = "tags"
        const val EXTRA_NOTE = "note"
        
        // Result keys for Tasker variables
        const val RESULT_SUCCESS = "success"
        const val RESULT_ERROR = "error"
        const val RESULT_DNP_COUNT = "dnp_count"
        const val RESULT_CONNECTED_SERVICES = "connected_services"
        const val RESULT_LAST_ENFORCEMENT = "last_enforcement"
        const val RESULT_ARTIST_NAME = "artist_name"
        const val RESULT_ARTIST_ID = "artist_id"
    }
    
    override fun onReceive(context: Context, intent: Intent) {
        Log.d(TAG, "Received Tasker action: ${intent.action}")
        
        val pendingResult = goAsync()
        val scope = CoroutineScope(Dispatchers.IO)
        
        scope.launch {
            try {
                when (intent.action) {
                    ACTION_ADD_ARTIST -> handleAddArtist(context, intent)
                    ACTION_REMOVE_ARTIST -> handleRemoveArtist(context, intent)
                    ACTION_CHECK_STATUS -> handleCheckStatus(context, intent)
                    ACTION_SKIP_TRACK -> handleSkipTrack(context, intent)
                    else -> {
                        Log.w(TAG, "Unknown action: ${intent.action}")
                        setTaskerResult(context, false, "Unknown action")
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error handling Tasker action", e)
                setTaskerResult(context, false, e.message ?: "Unknown error")
            } finally {
                pendingResult.finish()
            }
        }
    }
    
    private suspend fun handleAddArtist(context: Context, intent: Intent) {
        val artistName = intent.getStringExtra(EXTRA_ARTIST_NAME)
        if (artistName.isNullOrBlank()) {
            setTaskerResult(context, false, "Artist name is required")
            return
        }
        
        val tags = intent.getStringArrayExtra(EXTRA_TAGS)?.toList() ?: listOf("tasker-added")
        val note = intent.getStringExtra(EXTRA_NOTE) ?: "Added via Tasker"
        
        try {
            // Search for artist first
            val searchResults = apiClient.searchArtists(artistName, limit = 1)
            if (searchResults.artists.isEmpty()) {
                setTaskerResult(context, false, "Artist not found: $artistName")
                return
            }
            
            val artist = searchResults.artists.first()
            
            // Add to DNP list
            apiClient.addArtistToDNP(artist.id, tags, note)
            
            // Set success result with artist info
            val resultBundle = Bundle().apply {
                putBoolean(RESULT_SUCCESS, true)
                putString(RESULT_ARTIST_NAME, artist.name)
                putString(RESULT_ARTIST_ID, artist.id)
            }
            setTaskerResult(context, resultBundle)
            
            Log.i(TAG, "Successfully added artist to DNP: ${artist.name}")
            
        } catch (e: Exception) {
            Log.e(TAG, "Failed to add artist to DNP", e)
            setTaskerResult(context, false, "Failed to add artist: ${e.message}")
        }
    }
    
    private suspend fun handleRemoveArtist(context: Context, intent: Intent) {
        val artistName = intent.getStringExtra(EXTRA_ARTIST_NAME)
        val artistId = intent.getStringExtra(EXTRA_ARTIST_ID)
        
        if (artistName.isNullOrBlank() && artistId.isNullOrBlank()) {
            setTaskerResult(context, false, "Artist name or ID is required")
            return
        }
        
        try {
            val targetArtistId = if (!artistId.isNullOrBlank()) {
                artistId
            } else {
                // Find artist in DNP list by name
                val dnpList = apiClient.getDNPList()
                val matchingArtist = dnpList.artists.find { 
                    it.name.equals(artistName, ignoreCase = true) 
                }
                matchingArtist?.id ?: run {
                    setTaskerResult(context, false, "Artist not found in DNP list: $artistName")
                    return
                }
            }
            
            // Remove from DNP list
            apiClient.removeArtistFromDNP(targetArtistId)
            
            // Set success result
            val resultBundle = Bundle().apply {
                putBoolean(RESULT_SUCCESS, true)
                putString(RESULT_ARTIST_ID, targetArtistId)
                if (!artistName.isNullOrBlank()) {
                    putString(RESULT_ARTIST_NAME, artistName)
                }
            }
            setTaskerResult(context, resultBundle)
            
            Log.i(TAG, "Successfully removed artist from DNP: $targetArtistId")
            
        } catch (e: Exception) {
            Log.e(TAG, "Failed to remove artist from DNP", e)
            setTaskerResult(context, false, "Failed to remove artist: ${e.message}")
        }
    }
    
    private suspend fun handleCheckStatus(context: Context, intent: Intent) {
        try {
            val status = apiClient.getEnforcementStatus()
            
            val resultBundle = Bundle().apply {
                putBoolean(RESULT_SUCCESS, true)
                putInt(RESULT_DNP_COUNT, status.dnpCount)
                putStringArray(RESULT_CONNECTED_SERVICES, status.connectedServices.toTypedArray())
                putString(RESULT_LAST_ENFORCEMENT, status.lastEnforcement)
            }
            setTaskerResult(context, resultBundle)
            
            Log.i(TAG, "Successfully retrieved enforcement status")
            
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get enforcement status", e)
            setTaskerResult(context, false, "Failed to get status: ${e.message}")
        }
    }
    
    private suspend fun handleSkipTrack(context: Context, intent: Intent) {
        try {
            // This would integrate with media session controls
            // For now, we'll just log the action
            Log.i(TAG, "Skip track action received")
            
            val resultBundle = Bundle().apply {
                putBoolean(RESULT_SUCCESS, true)
            }
            setTaskerResult(context, resultBundle)
            
        } catch (e: Exception) {
            Log.e(TAG, "Failed to skip track", e)
            setTaskerResult(context, false, "Failed to skip track: ${e.message}")
        }
    }
    
    private fun setTaskerResult(context: Context, success: Boolean, error: String) {
        val bundle = Bundle().apply {
            putBoolean(RESULT_SUCCESS, success)
            if (!success) {
                putString(RESULT_ERROR, error)
            }
        }
        setTaskerResult(context, bundle)
    }
    
    private fun setTaskerResult(context: Context, resultBundle: Bundle) {
        TaskerResultHelper.setResult(context, resultBundle)
    }
}