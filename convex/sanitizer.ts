import { ConvexError, v } from "convex/values";
import { action, mutation, query } from "./_generated/server";
import { requireCurrentUser } from "./lib/auth";

const _computeGrade = query({
  args: {
    provider: v.string(),
    playlistId: v.string(),
  },
  handler: async (ctx, args) => {
    const allOffenses = await ctx.db.query("artistOffenses").collect();
    const offendingArtistIds = new Set(
      allOffenses.map((o) => o.artistId as string),
    );

    const allTracks = await ctx.db.query("userLibraryTracks").collect();
    const tracks = allTracks.filter(
      (t) =>
        t.provider === args.provider && t.playlistName === args.playlistId,
    );

    const totalTracks = tracks.length;
    const flagged = tracks.filter(
      (t) => t.artistId && offendingArtistIds.has(t.artistId as string),
    );

    const ratio =
      totalTracks > 0 ? (totalTracks - flagged.length) / totalTracks : 1;
    let grade = "A+";
    if (ratio < 0.5) grade = "F";
    else if (ratio < 0.6) grade = "D";
    else if (ratio < 0.7) grade = "C";
    else if (ratio < 0.8) grade = "B";
    else if (ratio < 0.95) grade = "A";

    return {
      playlist_id: args.playlistId,
      provider: args.provider,
      grade,
      total_tracks: totalTracks,
      flagged_tracks: flagged.length,
      clean_ratio: ratio,
      flagged_artists: [
        ...new Set(flagged.map((t) => t.artistName).filter(Boolean)),
      ],
    };
  },
});

export { _computeGrade as computeGrade };

export const gradePlaylist = action({
  args: {
    provider: v.string(),
    playlistId: v.string(),
  },
  handler: async (ctx, args) => {
    const grade = await ctx.runQuery(
      "sanitizer:computeGrade" as any,
      { provider: args.provider, playlistId: args.playlistId },
    );
    return grade;
  },
});

export const suggestReplacements = action({
  args: {
    provider: v.string(),
    playlistId: v.string(),
    flaggedTrackIds: v.optional(v.array(v.string())),
  },
  handler: async (ctx, args) => {
    // Fetch playlist data and identify flagged vs. clean tracks
    const playlistData = await ctx.runQuery(
      "sanitizer:_getPlaylistAnalysis" as any,
      {
        provider: args.provider,
        playlistId: args.playlistId,
        flaggedTrackIds: args.flaggedTrackIds,
      },
    );

    if (!playlistData || playlistData.flaggedTracks.length === 0) {
      return {
        playlist_id: args.playlistId,
        provider: args.provider,
        suggestions: [],
        message: "No flagged tracks found in this playlist.",
      };
    }

    // Get the user's connection for the access token
    const connection = await ctx.runQuery(
      "sanitizer:_getConnection" as any,
      { provider: args.provider },
    );

    if (!connection || !connection.accessToken) {
      return {
        playlist_id: args.playlistId,
        provider: args.provider,
        suggestions: [],
        message: `No active ${args.provider} connection found. Connect your account to get replacement suggestions.`,
      };
    }

    const accessToken = connection.accessToken;
    const suggestions: Array<{
      flagged_track_id: string;
      flagged_track_name: string;
      flagged_artist_name: string;
      replacements: Array<{
        track_id: string;
        track_name: string;
        artist_name: string;
        album_name: string;
        preview_url: string | null;
        external_url: string | null;
      }>;
    }> = [];

    // Use clean tracks as seed material for recommendations
    const seedTrackIds = playlistData.cleanTrackProviderIds.slice(0, 5);
    const seedArtists = playlistData.cleanArtistNames.slice(0, 3);

    // Look up Spotify seed artist IDs if possible
    const seedArtistIds: string[] = [];
    if (args.provider === "spotify" && seedArtists.length > 0) {
      for (const artistName of seedArtists.slice(0, 2)) {
        try {
          const searchResp = await fetch(
            `https://api.spotify.com/v1/search?q=${encodeURIComponent(artistName)}&type=artist&limit=1`,
            { headers: { Authorization: `Bearer ${accessToken}` } },
          );
          if (searchResp.ok) {
            const data = (await searchResp.json()) as any;
            const artist = data?.artists?.items?.[0];
            if (artist?.id) seedArtistIds.push(artist.id);
          }
        } catch {
          // Best effort
        }
      }
    }

    // Collect IDs of flagged artists so we can exclude them from recommendations
    const flaggedArtistNames = new Set(
      playlistData.flaggedTracks.map(
        (t: { artistName: string }) => t.artistName,
      ),
    );

    if (args.provider === "spotify") {
      // Build recommendation request seeds
      const seedParams = new URLSearchParams();
      if (seedTrackIds.length > 0) {
        seedParams.set("seed_tracks", seedTrackIds.slice(0, 3).join(","));
      }
      if (seedArtistIds.length > 0) {
        seedParams.set("seed_artists", seedArtistIds.slice(0, 2).join(","));
      }

      // We need at least one seed
      if (!seedParams.has("seed_tracks") && !seedParams.has("seed_artists")) {
        // Fall back to using a genre seed if available
        seedParams.set("seed_genres", "pop");
      }

      const totalNeeded = Math.min(playlistData.flaggedTracks.length * 3, 100);
      seedParams.set("limit", String(totalNeeded));

      try {
        const recResp = await fetch(
          `https://api.spotify.com/v1/recommendations?${seedParams.toString()}`,
          { headers: { Authorization: `Bearer ${accessToken}` } },
        );

        if (recResp.ok) {
          const recData = (await recResp.json()) as any;
          const recommendedTracks: Array<{
            id: string;
            name: string;
            artists: Array<{ name: string }>;
            album: { name: string };
            preview_url: string | null;
            external_urls: { spotify?: string };
          }> = recData?.tracks ?? [];

          // Filter out tracks from flagged artists
          const cleanRecs = recommendedTracks.filter(
            (rt) =>
              !rt.artists.some((a) => flaggedArtistNames.has(a.name)),
          );

          // Distribute recommendations across flagged tracks
          let recIdx = 0;
          for (const flagged of playlistData.flaggedTracks) {
            const trackRecs: typeof suggestions[0]["replacements"] = [];
            for (let i = 0; i < 3 && recIdx < cleanRecs.length; i++, recIdx++) {
              const rec = cleanRecs[recIdx];
              trackRecs.push({
                track_id: rec.id,
                track_name: rec.name,
                artist_name: rec.artists.map((a) => a.name).join(", "),
                album_name: rec.album.name,
                preview_url: rec.preview_url ?? null,
                external_url: rec.external_urls?.spotify ?? null,
              });
            }
            suggestions.push({
              flagged_track_id: flagged.providerTrackId,
              flagged_track_name: flagged.trackName,
              flagged_artist_name: flagged.artistName,
              replacements: trackRecs,
            });
          }
        } else {
          const errText = await recResp.text();
          return {
            playlist_id: args.playlistId,
            provider: args.provider,
            suggestions: [],
            message: `Spotify recommendations API error: ${recResp.status} ${errText.substring(0, 200)}`,
          };
        }
      } catch (err: any) {
        return {
          playlist_id: args.playlistId,
          provider: args.provider,
          suggestions: [],
          message: `Failed to fetch recommendations: ${err.message ?? "Unknown error"}`,
        };
      }
    } else {
      // For non-Spotify providers, return empty suggestions with a helpful message
      return {
        playlist_id: args.playlistId,
        provider: args.provider,
        suggestions: [],
        message: `Replacement suggestions are currently only supported for Spotify. ${args.provider} support is coming soon.`,
      };
    }

    return {
      playlist_id: args.playlistId,
      provider: args.provider,
      suggestions,
      message:
        suggestions.length > 0
          ? `Found ${suggestions.length} replacement suggestion(s) for flagged tracks.`
          : "Could not find suitable replacements. Try adjusting your playlist.",
    };
  },
});

export const updatePlan = mutation({
  args: {
    planId: v.string(),
    acceptedReplacements: v.optional(v.array(v.string())),
    rejectedReplacements: v.optional(v.array(v.string())),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    return {
      plan_id: args.planId,
      accepted: args.acceptedReplacements?.length ?? 0,
      rejected: args.rejectedReplacements?.length ?? 0,
      status: "updated",
    };
  },
});

export const publishPlan = action({
  args: {
    planId: v.string(),
  },
  handler: async (_ctx, args) => {
    return {
      plan_id: args.planId,
      status: "published",
      message:
        "Plan published. Changes will be applied once provider API integration is complete.",
    };
  },
});

// ---------------------------------------------------------------------------
// Internal queries
// ---------------------------------------------------------------------------

export const _getPlaylistAnalysis = query({
  args: {
    provider: v.string(),
    playlistId: v.string(),
    flaggedTrackIds: v.optional(v.array(v.string())),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);

    // Get all offending artist IDs
    const allOffenses = await ctx.db.query("artistOffenses").collect();
    const offendingArtistIds = new Set(
      allOffenses.map((o) => o.artistId as string),
    );

    // Get user-blocked artist IDs
    const userBlocks = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();
    const blockedArtistIds = new Set(
      userBlocks.map((b) => b.artistId as string),
    );

    const allFlagged = new Set([...offendingArtistIds, ...blockedArtistIds]);

    // Get tracks for the playlist
    const allTracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )
      .collect();

    const playlistTracks = allTracks.filter(
      (t) => t.playlistName === args.playlistId,
    );

    if (playlistTracks.length === 0) return null;

    const flaggedTracks: Array<{
      providerTrackId: string;
      trackName: string;
      artistName: string;
      artistId: string;
    }> = [];

    const cleanTrackProviderIds: string[] = [];
    const cleanArtistNames: string[] = [];
    const cleanArtistNamesSet = new Set<string>();

    for (const track of playlistTracks) {
      const aid = track.artistId as string | undefined;
      const isFlaggedById = aid ? allFlagged.has(aid) : false;
      const isFlaggedExplicitly = args.flaggedTrackIds
        ? args.flaggedTrackIds.includes(track.providerTrackId)
        : false;

      if (isFlaggedById || isFlaggedExplicitly) {
        flaggedTracks.push({
          providerTrackId: track.providerTrackId,
          trackName: track.trackName ?? "Unknown",
          artistName: track.artistName ?? "Unknown",
          artistId: aid ?? "",
        });
      } else {
        cleanTrackProviderIds.push(track.providerTrackId);
        if (track.artistName && !cleanArtistNamesSet.has(track.artistName)) {
          cleanArtistNamesSet.add(track.artistName);
          cleanArtistNames.push(track.artistName);
        }
      }
    }

    return {
      flaggedTracks,
      cleanTrackProviderIds,
      cleanArtistNames,
      totalTracks: playlistTracks.length,
    };
  },
});

export const _getConnection = query({
  args: {
    provider: v.string(),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);

    const connection = await ctx.db
      .query("providerConnections")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )
      .unique();

    if (!connection || connection.status !== "active") {
      return null;
    }

    return {
      connectionId: connection._id,
      accessToken: connection.encryptedAccessToken ?? null,
      refreshToken: connection.encryptedRefreshToken ?? null,
      expiresAt: connection.expiresAt ?? null,
    };
  },
});
