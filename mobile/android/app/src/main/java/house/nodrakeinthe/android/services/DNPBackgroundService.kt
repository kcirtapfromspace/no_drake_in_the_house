package house.nodrakeinthe.android.services

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import house.nodrakeinthe.android.R
import house.nodrakeinthe.android.activities.MainActivity
import house.nodrakeinthe.android.utils.BloomFilterManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.util.concurrent.TimeUnit

class DNPBackgroundService : Service() {
    
    private var serviceJob: Job? = null
    private val serviceScope = CoroutineScope(Dispatchers.IO)
    private lateinit var bloomFilterManager: BloomFilterManager
    
    companion object {
        private const val TAG = "DNPBackgroundService"
        private const val CHANNEL_ID = "dnp_background_service"
        private const val NOTIFICATION_ID = 1000
        private val FILTER_REFRESH_INTERVAL = TimeUnit.HOURS.toMillis(6) // 6 hours
    }
    
    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "DNP Background Service created")
        
        bloomFilterManager = BloomFilterManager(this)
        createNotificationChannel()
    }
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "DNP Background Service started")
        
        startForeground(NOTIFICATION_ID, createNotification())
        
        // Start background tasks
        serviceJob = serviceScope.launch {
            runBackgroundTasks()
        }
        
        return START_STICKY // Restart if killed
    }
    
    override fun onBind(intent: Intent?): IBinder? = null
    
    override fun onDestroy() {
        super.onDestroy()
        Log.d(TAG, "DNP Background Service destroyed")
        
        serviceJob?.cancel()
    }
    
    private suspend fun runBackgroundTasks() {
        while (true) {
            try {
                // Refresh bloom filter periodically
                bloomFilterManager.refreshFilter()
                
                Log.d(TAG, "Background tasks completed")
                
                // Wait for next refresh interval
                delay(FILTER_REFRESH_INTERVAL)
                
            } catch (e: Exception) {
                Log.e(TAG, "Error in background tasks", e)
                
                // Wait a shorter time before retrying on error
                delay(TimeUnit.MINUTES.toMillis(30))
            }
        }
    }
    
    private fun createNotification(): Notification {
        val intent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this, 0, intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("DNP Manager")
            .setContentText("Monitoring music playback")
            .setSmallIcon(R.drawable.ic_notification)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setShowWhen(false)
            .build()
    }
    
    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "DNP Background Service",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Keeps DNP Manager running in the background"
                setShowBadge(false)
            }
            
            val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            notificationManager.createNotificationChannel(channel)
        }
    }
}