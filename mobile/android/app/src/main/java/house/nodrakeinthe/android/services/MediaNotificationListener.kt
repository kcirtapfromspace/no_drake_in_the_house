package house.nodrakeinthe.android.services

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.media.MediaMetadata
import android.media.session.MediaController
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.Build
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import android.util.Log
import androidx.core.app.NotificationCompat
import house.nodrakeinthe.android.R
import house.nodrakeinthe.android.api.DNPApiClient
import house.nodrakeinthe.android.receivers.MediaActionReceiver
import house.nodrakeinthe.android.utils.BloomFilterManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class MediaNotificationListener : NotificationListenerService() {
    
    private val apiClient = DNPApiClient()
    private val bloomFilterManager = BloomFilterManager(this)
    private val serviceScope = CoroutineScope(Dispatchers.IO)
    private var currentMediaController: MediaController? = null
    
    companion object {
        private const val TAG = "MediaNotificationListener"
        private const val CHANNEL_ID = "dnp_media_controls"
        private const val NOTIFICATION_ID = 1001
        
        // Supported music apps
        private val SUPPORTED_PACKAGES = setOf(
            "com.spotify.music",
            "com.apple.android.music",
            "com.google.android.apps.youtube.music",
            "com.aspiro.tidal"
        )
    }
    
    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        bloomFilterManager.loadDNPFilter()
    }
    
    override fun onNotificationPosted(sbn: StatusBarNotification) {
        super.onNotificationPosted(sbn)
        
        if (!SUPPORTED_PACKAGES.contains(sbn.packageName)) {
            return
        }
        
        val notification = sbn.notification
        val extras = notification.extras
        
        // Extract media session token
        val mediaSession = extras.getParcelable<MediaSession.Token>(
            Notification.EXTRA_MEDIA_SESSION
        )
        
        mediaSession?.let { token ->
            try {
                currentMediaController = MediaController(this, token)
                setupMediaController()
            } catch (e: Exception) {
                Log.e(TAG, "Failed to create MediaController", e)
            }
        }
    }
    
    private fun setupMediaController() {
        currentMediaController?.let { controller ->
            controller.registerCallback(object : MediaController.Callback() {
                override fun onMetadataChanged(metadata: MediaMetadata?) {
                    super.onMetadataChanged(metadata)
                    metadata?.let { checkCurrentTrack(it, controller) }
                }
                
                override fun onPlaybackStateChanged(state: PlaybackState?) {
                    super.onPlaybackStateChanged(state)
                    if (state?.state == PlaybackState.STATE_PLAYING) {
                        currentMediaController?.metadata?.let { metadata ->
                            checkCurrentTrack(metadata, controller)
                        }
                    }
                }
            })
            
            // Check current track immediately
            controller.metadata?.let { metadata ->
                checkCurrentTrack(metadata, controller)
            }
        }
    }
    
    private fun checkCurrentTrack(metadata: MediaMetadata, controller: MediaController) {
        val artist = metadata.getString(MediaMetadata.METADATA_KEY_ARTIST) ?: return
        val title = metadata.getString(MediaMetadata.METADATA_KEY_TITLE) ?: "Unknown"
        val album = metadata.getString(MediaMetadata.METADATA_KEY_ALBUM) ?: ""
        
        Log.d(TAG, "Checking track: $title by $artist")
        
        // Quick bloom filter check
        if (bloomFilterManager.mightContain(artist)) {
            serviceScope.launch {
                try {
                    val isBlocked = apiClient.isArtistBlocked(artist)
                    if (isBlocked) {
                        handleBlockedTrack(artist, title, album, controller)
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to check if artist is blocked", e)
                }
            }
        }
    }
    
    private fun handleBlockedTrack(
        artist: String,
        title: String,
        album: String,
        controller: MediaController
    ) {
        Log.i(TAG, "Blocked track detected: $title by $artist")
        
        // Skip the track
        controller.transportControls.skipToNext()
        
        // Show notification with controls
        showBlockedTrackNotification(artist, title, album)
    }
    
    private fun showBlockedTrackNotification(artist: String, title: String, album: String) {
        val skipIntent = Intent(this, MediaActionReceiver::class.java).apply {
            action = MediaActionReceiver.ACTION_SKIP_CONFIRMED
            putExtra("artist", artist)
            putExtra("title", title)
        }
        val skipPendingIntent = PendingIntent.getBroadcast(
            this, 0, skipIntent, 
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        
        val undoIntent = Intent(this, MediaActionReceiver::class.java).apply {
            action = MediaActionReceiver.ACTION_UNDO_SKIP
            putExtra("artist", artist)
            putExtra("title", title)
        }
        val undoPendingIntent = PendingIntent.getBroadcast(
            this, 1, undoIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        
        val removeFromDNPIntent = Intent(this, MediaActionReceiver::class.java).apply {
            action = MediaActionReceiver.ACTION_REMOVE_FROM_DNP
            putExtra("artist", artist)
        }
        val removePendingIntent = PendingIntent.getBroadcast(
            this, 2, removeFromDNPIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        
        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_skip)
            .setContentTitle("Skipped blocked track")
            .setContentText("$title by $artist")
            .setStyle(NotificationCompat.BigTextStyle()
                .bigText("Skipped \"$title\" by $artist${if (album.isNotEmpty()) " from $album" else ""}"))
            .addAction(R.drawable.ic_undo, "Undo", undoPendingIntent)
            .addAction(R.drawable.ic_remove, "Unblock Artist", removePendingIntent)
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setCategory(NotificationCompat.CATEGORY_STATUS)
            .build()
        
        val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        notificationManager.notify(NOTIFICATION_ID, notification)
    }
    
    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "DNP Media Controls",
                NotificationManager.IMPORTANCE_DEFAULT
            ).apply {
                description = "Notifications for blocked track actions"
                setShowBadge(false)
            }
            
            val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            notificationManager.createNotificationChannel(channel)
        }
    }
    
    override fun onDestroy() {
        super.onDestroy()
        currentMediaController?.unregisterCallback(null)
    }
}