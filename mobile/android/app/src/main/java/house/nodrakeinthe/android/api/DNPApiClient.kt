package house.nodrakeinthe.android.api

import android.content.Context
import android.content.SharedPreferences
import android.util.Log
import com.google.gson.Gson
import com.google.gson.annotations.SerializedName
import house.nodrakeinthe.android.models.Artist
import house.nodrakeinthe.android.models.ArtistSearchResponse
import house.nodrakeinthe.android.models.DNPListResponse
import house.nodrakeinthe.android.models.EnforcementStatus
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.IOException
import java.util.concurrent.TimeUnit

class DNPApiClient(private val context: Context? = null) {
    
    private val client = OkHttpClient.Builder()
        .connectTimeout(30, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .writeTimeout(30, TimeUnit.SECONDS)
        .build()
    
    private val gson = Gson()
    
    companion object {
        private const val TAG = "DNPApiClient"
        private const val DEFAULT_BASE_URL = "https://api.nodrakeinthe.house"
        private const val PREFS_NAME = "dnp_api_prefs"
        private const val KEY_BASE_URL = "base_url"
        private const val KEY_API_TOKEN = "api_token"
    }
    
    private val baseUrl: String
        get() = context?.let { ctx ->
            val prefs = ctx.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            prefs.getString(KEY_BASE_URL, DEFAULT_BASE_URL) ?: DEFAULT_BASE_URL
        } ?: DEFAULT_BASE_URL
    
    private val apiToken: String?
        get() = context?.let { ctx ->
            val prefs = ctx.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            prefs.getString(KEY_API_TOKEN, null)
        }
    
    suspend fun searchArtists(query: String, limit: Int = 10): ArtistSearchResponse = withContext(Dispatchers.IO) {
        val requestBody = gson.toJson(mapOf(
            "query" to query,
            "limit" to limit
        ))
        
        val request = Request.Builder()
            .url("$baseUrl/v1/artists/search")
            .post(requestBody.toRequestBody("application/json".toMediaType()))
            .addHeader("Authorization", "Bearer ${apiToken ?: ""}")
            .addHeader("Content-Type", "application/json")
            .build()
        
        val response = client.newCall(request).execute()
        if (!response.isSuccessful) {
            throw IOException("API request failed: ${response.code} ${response.message}")
        }
        
        val responseBody = response.body?.string() ?: throw IOException("Empty response body")
        gson.fromJson(responseBody, ArtistSearchResponse::class.java)
    }
    
    suspend fun addArtistToDNP(artistId: String, tags: List<String>, note: String) = withContext(Dispatchers.IO) {
        val requestBody = gson.toJson(mapOf(
            "artist_id" to artistId,
            "tags" to tags,
            "note" to note
        ))
        
        val request = Request.Builder()
            .url("$baseUrl/v1/dnp/artists")
            .post(requestBody.toRequestBody("application/json".toMediaType()))
            .addHeader("Authorization", "Bearer ${apiToken ?: ""}")
            .addHeader("Content-Type", "application/json")
            .build()
        
        val response = client.newCall(request).execute()
        if (!response.isSuccessful) {
            throw IOException("Failed to add artist to DNP: ${response.code} ${response.message}")
        }
    }
    
    suspend fun removeArtistFromDNP(artistId: String) = withContext(Dispatchers.IO) {
        val request = Request.Builder()
            .url("$baseUrl/v1/dnp/artists/$artistId")
            .delete()
            .addHeader("Authorization", "Bearer ${apiToken ?: ""}")
            .build()
        
        val response = client.newCall(request).execute()
        if (!response.isSuccessful) {
            throw IOException("Failed to remove artist from DNP: ${response.code} ${response.message}")
        }
    }
    
    suspend fun getDNPList(): DNPListResponse = withContext(Dispatchers.IO) {
        val request = Request.Builder()
            .url("$baseUrl/v1/dnp/artists")
            .get()
            .addHeader("Authorization", "Bearer ${apiToken ?: ""}")
            .build()
        
        val response = client.newCall(request).execute()
        if (!response.isSuccessful) {
            throw IOException("Failed to get DNP list: ${response.code} ${response.message}")
        }
        
        val responseBody = response.body?.string() ?: throw IOException("Empty response body")
        gson.fromJson(responseBody, DNPListResponse::class.java)
    }
    
    suspend fun getEnforcementStatus(): EnforcementStatus = withContext(Dispatchers.IO) {
        val request = Request.Builder()
            .url("$baseUrl/v1/enforcement/status")
            .get()
            .addHeader("Authorization", "Bearer ${apiToken ?: ""}")
            .build()
        
        val response = client.newCall(request).execute()
        if (!response.isSuccessful) {
            throw IOException("Failed to get enforcement status: ${response.code} ${response.message}")
        }
        
        val responseBody = response.body?.string() ?: throw IOException("Empty response body")
        gson.fromJson(responseBody, EnforcementStatus::class.java)
    }
    
    suspend fun isArtistBlocked(artistName: String): Boolean = withContext(Dispatchers.IO) {
        try {
            val dnpList = getDNPList()
            return@withContext dnpList.artists.any { artist ->
                artist.name.equals(artistName, ignoreCase = true)
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to check if artist is blocked", e)
            return@withContext false
        }
    }
    
    fun setApiCredentials(baseUrl: String, token: String) {
        context?.let { ctx ->
            val prefs = ctx.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            prefs.edit()
                .putString(KEY_BASE_URL, baseUrl)
                .putString(KEY_API_TOKEN, token)
                .apply()
        }
    }
    
    fun clearApiCredentials() {
        context?.let { ctx ->
            val prefs = ctx.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            prefs.edit()
                .remove(KEY_API_TOKEN)
                .apply()
        }
    }
}