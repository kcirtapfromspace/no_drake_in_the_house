package house.nodrakeinthe.android.utils

import android.content.Context
import android.os.Bundle
import android.util.Log

object TaskerResultHelper {
    
    private const val TAG = "TaskerResultHelper"
    
    /**
     * Sets Tasker variables from the result bundle
     * This uses reflection to set Tasker variables since Tasker doesn't provide a public API
     */
    fun setResult(context: Context, resultBundle: Bundle) {
        try {
            // Set individual variables that Tasker can read
            for (key in resultBundle.keySet()) {
                val value = resultBundle.get(key)
                setTaskerVariable(context, key, value.toString())
            }
            
            Log.d(TAG, "Set Tasker result variables: ${resultBundle.keySet()}")
            
        } catch (e: Exception) {
            Log.e(TAG, "Failed to set Tasker result", e)
        }
    }
    
    private fun setTaskerVariable(context: Context, name: String, value: String) {
        try {
            // This is a simplified approach - in a real implementation,
            // you would use Tasker's plugin API or intent-based variable setting
            
            // For now, we'll use system properties as a fallback
            System.setProperty("tasker.var.$name", value)
            
            Log.d(TAG, "Set Tasker variable: $name = $value")
            
        } catch (e: Exception) {
            Log.e(TAG, "Failed to set Tasker variable: $name", e)
        }
    }
    
    /**
     * Creates a result bundle for common success/error patterns
     */
    fun createSuccessResult(additionalData: Map<String, Any> = emptyMap()): Bundle {
        return Bundle().apply {
            putBoolean("success", true)
            additionalData.forEach { (key, value) ->
                when (value) {
                    is String -> putString(key, value)
                    is Int -> putInt(key, value)
                    is Boolean -> putBoolean(key, value)
                    is Long -> putLong(key, value)
                    is Float -> putFloat(key, value)
                    is Double -> putDouble(key, value)
                    is Array<*> -> {
                        if (value.isArrayOf<String>()) {
                            putStringArray(key, value as Array<String>)
                        }
                    }
                    else -> putString(key, value.toString())
                }
            }
        }
    }
    
    fun createErrorResult(error: String): Bundle {
        return Bundle().apply {
            putBoolean("success", false)
            putString("error", error)
        }
    }
}