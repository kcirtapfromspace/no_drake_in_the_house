package house.nodrakeinthe.android.receivers

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import house.nodrakeinthe.android.services.DNPBackgroundService

class BootReceiver : BroadcastReceiver() {
    
    companion object {
        private const val TAG = "BootReceiver"
    }
    
    override fun onReceive(context: Context, intent: Intent) {
        Log.d(TAG, "Received boot action: ${intent.action}")
        
        when (intent.action) {
            Intent.ACTION_BOOT_COMPLETED,
            Intent.ACTION_MY_PACKAGE_REPLACED,
            Intent.ACTION_PACKAGE_REPLACED -> {
                startBackgroundService(context)
            }
        }
    }
    
    private fun startBackgroundService(context: Context) {
        try {
            val serviceIntent = Intent(context, DNPBackgroundService::class.java)
            context.startForegroundService(serviceIntent)
            Log.i(TAG, "Started DNP background service")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start background service", e)
        }
    }
}