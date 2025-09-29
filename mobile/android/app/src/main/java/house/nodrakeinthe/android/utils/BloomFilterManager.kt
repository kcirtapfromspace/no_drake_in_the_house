package house.nodrakeinthe.android.utils

import android.content.Context
import android.util.Log
import com.google.common.hash.BloomFilter
import com.google.common.hash.Funnels
import house.nodrakeinthe.android.api.DNPApiClient
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.IOException
import java.nio.charset.Charset

class BloomFilterManager(private val context: Context) {
    
    private var bloomFilter: BloomFilter<String>? = null
    private val apiClient = DNPApiClient(context)
    private val scope = CoroutineScope(Dispatchers.IO)
    
    companion object {
        private const val TAG = "BloomFilterManager"
        private const val BLOOM_FILTER_FILE = "dnp_bloom_filter.dat"
        private const val EXPECTED_INSERTIONS = 10000
        private const val FALSE_POSITIVE_PROBABILITY = 0.01
    }
    
    init {
        loadDNPFilter()
    }
    
    fun loadDNPFilter() {
        scope.launch {
            try {
                // Try to load existing filter from file
                if (loadFilterFromFile()) {
                    Log.d(TAG, "Loaded bloom filter from file")
                    return@launch
                }
                
                // If no file exists, create new filter from API
                refreshFilterFromAPI()
                
            } catch (e: Exception) {
                Log.e(TAG, "Failed to load DNP filter", e)
                createEmptyFilter()
            }
        }
    }
    
    private suspend fun refreshFilterFromAPI() {
        try {
            val dnpList = apiClient.getDNPList()
            
            val newFilter = BloomFilter.create(
                Funnels.stringFunnel(Charset.defaultCharset()),
                EXPECTED_INSERTIONS,
                FALSE_POSITIVE_PROBABILITY
            )
            
            // Add all artists to the filter
            dnpList.artists.forEach { artist ->
                newFilter.put(artist.name.lowercase())
                // Also add any aliases if available
                artist.metadata?.get("aliases")?.let { aliases ->
                    if (aliases is List<*>) {
                        aliases.forEach { alias ->
                            if (alias is String) {
                                newFilter.put(alias.lowercase())
                            }
                        }
                    }
                }
            }
            
            bloomFilter = newFilter
            saveFilterToFile()
            
            Log.i(TAG, "Refreshed bloom filter with ${dnpList.artists.size} artists")
            
        } catch (e: Exception) {
            Log.e(TAG, "Failed to refresh filter from API", e)
            createEmptyFilter()
        }
    }
    
    private fun createEmptyFilter() {
        bloomFilter = BloomFilter.create(
            Funnels.stringFunnel(Charset.defaultCharset()),
            EXPECTED_INSERTIONS,
            FALSE_POSITIVE_PROBABILITY
        )
        Log.d(TAG, "Created empty bloom filter")
    }
    
    private fun loadFilterFromFile(): Boolean {
        return try {
            val file = context.getFileStreamPath(BLOOM_FILTER_FILE)
            if (!file.exists()) {
                return false
            }
            
            FileInputStream(file).use { fis ->
                bloomFilter = BloomFilter.readFrom(fis, Funnels.stringFunnel(Charset.defaultCharset()))
            }
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to load filter from file", e)
            false
        }
    }
    
    private fun saveFilterToFile() {
        try {
            bloomFilter?.let { filter ->
                FileOutputStream(context.getFileStreamPath(BLOOM_FILTER_FILE)).use { fos ->
                    filter.writeTo(fos)
                }
                Log.d(TAG, "Saved bloom filter to file")
            }
        } catch (e: IOException) {
            Log.e(TAG, "Failed to save filter to file", e)
        }
    }
    
    fun mightContain(artistName: String): Boolean {
        return bloomFilter?.mightContain(artistName.lowercase()) ?: false
    }
    
    fun addArtist(artistName: String) {
        bloomFilter?.put(artistName.lowercase())
        saveFilterToFile()
    }
    
    fun refreshFilter() {
        scope.launch {
            refreshFilterFromAPI()
        }
    }
    
    fun getApproximateElementCount(): Long {
        return bloomFilter?.approximateElementCount() ?: 0
    }
    
    fun getExpectedFalsePositiveProbability(): Double {
        return bloomFilter?.expectedFpp() ?: 0.0
    }
}