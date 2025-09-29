package house.nodrakeinthe.android.services

import android.content.Intent
import android.graphics.drawable.Icon
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import android.util.Log
import house.nodrakeinthe.android.R
import house.nodrakeinthe.android.activities.MainActivity
import house.nodrakeinthe.android.api.DNPApiClient
import house.nodrakeinthe.android.models.EnforcementStatus
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class DNPQuickSettingsTile : TileService() {
    
    private val apiClient = DNPApiClient()
    private val serviceScope = CoroutineScope(Dispatchers.IO)
    
    companion object {
        private const val TAG = "DNPQuickSettingsTile"
    }
    
    override fun onStartListening() {
        super.onStartListening()
        updateTileState()
    }
    
    override fun onClick() {
        super.onClick()
        
        // Toggle between showing status and opening app
        if (qsTile.state == Tile.STATE_ACTIVE) {
            // Open main app
            val intent = Intent(this, MainActivity::class.java).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
            }
            startActivityAndCollapse(intent)
        } else {
            // Refresh status
            updateTileState()
        }
    }
    
    private fun updateTileState() {
        serviceScope.launch {
            try {
                val status = apiClient.getEnforcementStatus()
                withContext(Dispatchers.Main) {
                    updateTile(status)
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to fetch enforcement status", e)
                withContext(Dispatchers.Main) {
                    updateTileError()
                }
            }
        }
    }
    
    private fun updateTile(status: EnforcementStatus) {
        qsTile?.let { tile ->
            tile.state = if (status.connectedServices.isNotEmpty()) {
                Tile.STATE_ACTIVE
            } else {
                Tile.STATE_INACTIVE
            }
            
            tile.label = "DNP: ${status.dnpCount} artists"
            tile.contentDescription = "Do Not Play list with ${status.dnpCount} artists, " +
                    "${status.connectedServices.size} services connected"
            
            // Set subtitle with last enforcement info
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.Q) {
                tile.subtitle = "Last: ${status.lastEnforcement}"
            }
            
            // Update icon based on status
            val iconRes = if (status.connectedServices.isNotEmpty()) {
                R.drawable.ic_block_active
            } else {
                R.drawable.ic_block_inactive
            }
            tile.icon = Icon.createWithResource(this, iconRes)
            
            tile.updateTile()
        }
    }
    
    private fun updateTileError() {
        qsTile?.let { tile ->
            tile.state = Tile.STATE_UNAVAILABLE
            tile.label = "DNP: Error"
            tile.contentDescription = "Failed to load DNP status"
            
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.Q) {
                tile.subtitle = "Tap to retry"
            }
            
            tile.icon = Icon.createWithResource(this, R.drawable.ic_error)
            tile.updateTile()
        }
    }
}