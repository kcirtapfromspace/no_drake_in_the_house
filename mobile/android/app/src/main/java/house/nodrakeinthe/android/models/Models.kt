package house.nodrakeinthe.android.models

import com.google.gson.annotations.SerializedName

data class Artist(
    val id: String,
    val name: String,
    @SerializedName("external_ids")
    val externalIds: Map<String, String>? = null,
    val metadata: Map<String, Any>? = null
)

data class ArtistSearchResponse(
    val artists: List<Artist>
)

data class DNPListResponse(
    val artists: List<Artist>
)

data class EnforcementStatus(
    @SerializedName("dnp_count")
    val dnpCount: Int,
    @SerializedName("connected_services")
    val connectedServices: List<String>,
    @SerializedName("last_enforcement")
    val lastEnforcement: String,
    @SerializedName("service_details")
    val serviceDetails: Map<String, Any>? = null
)

data class AddArtistRequest(
    @SerializedName("artist_id")
    val artistId: String,
    val tags: List<String>,
    val note: String
)