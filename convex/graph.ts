import { ConvexError, v } from "convex/values";
import type { Doc, Id } from "./_generated/dataModel";
import { query } from "./_generated/server";
import { requireCurrentUser } from "./lib/auth";

type ArtistDoc = Doc<"artists">;
type CollaborationDoc = Doc<"artistCollaborations">;

function artistGenres(artist: ArtistDoc): string[] {
  const metadata = artist.metadata as Record<string, unknown> | undefined;
  return Array.isArray(metadata?.genres) ? (metadata.genres as string[]) : [];
}

function artistImage(artist: ArtistDoc): string | undefined {
  const metadata = artist.metadata as Record<string, unknown> | undefined;
  return typeof metadata?.image === "string" ? (metadata.image as string) : undefined;
}

function collaborationType(value: string | undefined) {
  switch (value) {
    case "producer":
    case "writer":
    case "remix":
      return value;
    default:
      return "feature" as const;
  }
}

async function loadArtists(ctx: any) {
  const artists = await ctx.db.query("artists").collect();
  return new Map<string, ArtistDoc>(artists.map((artist: ArtistDoc) => [artist._id, artist]));
}

async function loadCollaborations(ctx: any) {
  return (await ctx.db.query("artistCollaborations").collect()) as CollaborationDoc[];
}

function buildAdjacency(collaborations: CollaborationDoc[]) {
  const adjacency = new Map<string, CollaborationDoc[]>();

  for (const collaboration of collaborations) {
    const left = adjacency.get(collaboration.artistId1) ?? [];
    left.push(collaboration);
    adjacency.set(collaboration.artistId1, left);

    const right = adjacency.get(collaboration.artistId2) ?? [];
    right.push(collaboration);
    adjacency.set(collaboration.artistId2, right);
  }

  return adjacency;
}

async function blockedArtistIds(ctx: any, userId: Id<"users">) {
  const blocks = await ctx.db
    .query("userArtistBlocks")
    .withIndex("by_userId", (q: any) => q.eq("userId", userId))
    .collect();

  return new Set<string>(blocks.map((block: Doc<"userArtistBlocks">) => block.artistId));
}

function toGraphArtist(artist: ArtistDoc, isBlocked: boolean) {
  return {
    id: artist._id,
    name: artist.canonicalName,
    genres: artistGenres(artist),
    is_blocked: isBlocked,
    image_url: artistImage(artist),
  };
}

function edgeKey(source: string, target: string) {
  return [source, target].sort().join(":");
}

function trackList(collaboration: CollaborationDoc) {
  return Array.isArray(collaboration.recentTracks)
    ? (collaboration.recentTracks as unknown[])
    : [];
}

export const search = query({
  args: {
    query: v.string(),
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const matches = await ctx.db
      .query("artists")
      .withSearchIndex("search_canonicalName", (q: any) =>
        q.search("canonicalName", args.query),
      )
      .take(args.limit ?? 20);

    const blockedIds = await blockedArtistIds(ctx, user._id);

    return {
      artists: matches.map((artist: ArtistDoc) =>
        toGraphArtist(artist, blockedIds.has(artist._id)),
      ),
    };
  },
});

export const collaboratorsForArtist = query({
  args: {
    artistId: v.id("artists"),
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const [left, right, artists] = await Promise.all([
      ctx.db
        .query("artistCollaborations")
        .withIndex("by_artistId1", (q: any) => q.eq("artistId1", args.artistId))
        .collect(),
      ctx.db
        .query("artistCollaborations")
        .withIndex("by_artistId2", (q: any) => q.eq("artistId2", args.artistId))
        .collect(),
      loadArtists(ctx),
    ]);

    const collaborations = [...left, ...right].slice(0, args.limit ?? 20);
    const hydrated = collaborations.flatMap((collaboration) => {
      const collaboratorId =
        collaboration.artistId1 === args.artistId
          ? collaboration.artistId2
          : collaboration.artistId1;
      const collaborator = artists.get(collaboratorId);
      if (!collaborator) {
        return [];
      }

      const recentTracks = trackList(collaboration);
      if (recentTracks.length === 0) {
        return [
          {
            artist_id: collaborator._id,
            artist_name: collaborator.canonicalName,
            collab_type: collaborationType(collaboration.collaborationType),
          },
        ];
      }

      return recentTracks.map((track) => {
        const recentTrack =
          typeof track === "object" && track !== null
            ? (track as Record<string, unknown>)
            : {};

        return {
          artist_id: collaborator._id,
          artist_name: collaborator.canonicalName,
          track_id:
            typeof recentTrack.id === "string"
              ? (recentTrack.id as string)
              : undefined,
          track_title:
            typeof recentTrack.title === "string"
              ? (recentTrack.title as string)
              : undefined,
          collab_type: collaborationType(collaboration.collaborationType),
          year:
            typeof recentTrack.year === "number"
              ? (recentTrack.year as number)
              : undefined,
        };
      });
    });

    return { collaborators: hydrated };
  },
});

export const collaborators = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    const [left, right, artists] = await Promise.all([
      ctx.db
        .query("artistCollaborations")
        .withIndex("by_artistId1", (q: any) => q.eq("artistId1", args.artistId))
        .collect(),
      ctx.db
        .query("artistCollaborations")
        .withIndex("by_artistId2", (q: any) => q.eq("artistId2", args.artistId))
        .collect(),
      loadArtists(ctx),
    ]);

    return {
      collaborators: [...left, ...right]
        .map((collaboration) => {
          const collaboratorId =
            collaboration.artistId1 === args.artistId
              ? collaboration.artistId2
              : collaboration.artistId1;
          const artist = artists.get(collaboratorId);
          if (!artist) {
            return null;
          }

          return {
            id: artist._id,
            name: artist.canonicalName,
            image_url: artistImage(artist),
            collaboration_count: collaboration.collaborationCount ?? 1,
            is_flagged: false,
            status: artist.status ?? "clean",
            collaboration_type: collaboration.collaborationType,
            recent_tracks: trackList(collaboration)
              .map((track) => {
                const recentTrack =
                  typeof track === "object" && track !== null
                    ? (track as Record<string, unknown>)
                    : {};
                return typeof recentTrack.title === "string"
                  ? (recentTrack.title as string)
                  : null;
              })
              .filter(Boolean),
          };
        })
        .filter(Boolean),
    };
  },
});

export const artistNetwork = query({
  args: {
    artistId: v.id("artists"),
    depth: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const [artists, collaborations, blockedIds] = await Promise.all([
      loadArtists(ctx),
      loadCollaborations(ctx),
      blockedArtistIds(ctx, user._id),
    ]);

    const depth = Math.max(1, Math.min(args.depth ?? 2, 4));
    const adjacency = buildAdjacency(collaborations);
    const queue: Array<{ artistId: string; depth: number }> = [
      { artistId: args.artistId, depth: 0 },
    ];
    const seen = new Set<string>([args.artistId]);
    const networkEdges = new Map<string, any>();

    while (queue.length > 0) {
      const current = queue.shift()!;
      if (current.depth >= depth) {
        continue;
      }

      for (const collaboration of adjacency.get(current.artistId) ?? []) {
        const collaboratorId =
          collaboration.artistId1 === current.artistId
            ? collaboration.artistId2
            : collaboration.artistId1;

        networkEdges.set(edgeKey(current.artistId, collaboratorId), {
          source: current.artistId,
          target: collaboratorId,
          type: "collaborated_with",
          weight: collaboration.collaborationCount ?? 1,
          metadata: {
            collaboration_type: collaboration.collaborationType,
          },
        });

        if (!seen.has(collaboratorId)) {
          seen.add(collaboratorId);
          queue.push({ artistId: collaboratorId, depth: current.depth + 1 });
        }
      }
    }

    const nodes = Array.from(seen)
      .map((artistId) => artists.get(artistId))
      .filter(Boolean)
      .map((artist) => ({
        id: artist!._id,
        name: artist!.canonicalName,
        type: "artist" as const,
        is_blocked: blockedIds.has(artist!._id),
        genres: artistGenres(artist!),
        image_url: artistImage(artist!),
      }));

    return {
      nodes,
      edges: Array.from(networkEdges.values()),
      center_artist_id: args.artistId,
      depth,
    };
  },
});

export const pathBetweenArtists = query({
  args: {
    sourceArtistId: v.id("artists"),
    targetArtistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    const [artists, collaborations] = await Promise.all([
      loadArtists(ctx),
      loadCollaborations(ctx),
    ]);

    if (args.sourceArtistId === args.targetArtistId) {
      const artist = artists.get(args.sourceArtistId);
      if (!artist) {
        throw new ConvexError("Artist not found.");
      }

      return {
        path: [toGraphArtist(artist, false)],
        edges: [],
        total_distance: 0,
      };
    }

    const adjacency = buildAdjacency(collaborations);
    const queue = [args.sourceArtistId];
    const visited = new Set<string>([args.sourceArtistId]);
    const previous = new Map<string, { parent: string; edge: any }>();

    while (queue.length > 0 && !visited.has(args.targetArtistId)) {
      const current = queue.shift()!;
      for (const collaboration of adjacency.get(current) ?? []) {
        const next =
          collaboration.artistId1 === current
            ? collaboration.artistId2
            : collaboration.artistId1;

        if (visited.has(next)) {
          continue;
        }

        visited.add(next);
        previous.set(next, {
          parent: current,
          edge: {
            source: current,
            target: next,
            type: "collaborated_with",
            weight: collaboration.collaborationCount ?? 1,
          },
        });
        queue.push(next);
      }
    }

    if (!visited.has(args.targetArtistId)) {
      throw new ConvexError("No path found between the selected artists.");
    }

    const pathIds: string[] = [];
    const edges: any[] = [];
    let current = args.targetArtistId as string;

    while (current !== args.sourceArtistId) {
      pathIds.unshift(current);
      const step = previous.get(current);
      if (!step) {
        break;
      }
      edges.unshift(step.edge);
      current = step.parent;
    }

    pathIds.unshift(args.sourceArtistId);

    return {
      path: pathIds
        .map((artistId) => artists.get(artistId))
        .filter(Boolean)
        .map((artist) => toGraphArtist(artist!, false)),
      edges,
      total_distance: Math.max(pathIds.length - 1, 0),
    };
  },
});

export const blockedNetworkAnalysis = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const [artists, collaborations, blockedIds] = await Promise.all([
      loadArtists(ctx),
      loadCollaborations(ctx),
      blockedArtistIds(ctx, user._id),
    ]);

    const adjacency = buildAdjacency(collaborations);
    const atRiskMap = new Map<
      string,
      {
        artist: ReturnType<typeof toGraphArtist>;
        blocked_collaborators: number;
        risk_score: number;
      }
    >();

    for (const blockedId of blockedIds) {
      for (const collaboration of adjacency.get(blockedId) ?? []) {
        const collaboratorId =
          collaboration.artistId1 === blockedId
            ? collaboration.artistId2
            : collaboration.artistId1;

        if (blockedIds.has(collaboratorId)) {
          continue;
        }

        const collaborator = artists.get(collaboratorId);
        if (!collaborator) {
          continue;
        }

        const existing = atRiskMap.get(collaboratorId);
        const weight = collaboration.collaborationCount ?? 1;
        if (existing) {
          existing.blocked_collaborators += 1;
          existing.risk_score += weight;
        } else {
          atRiskMap.set(collaboratorId, {
            artist: toGraphArtist(collaborator, false),
            blocked_collaborators: 1,
            risk_score: weight,
          });
        }
      }
    }

    const blockedClusters: Array<{
      cluster_id: string;
      artists: ReturnType<typeof toGraphArtist>[];
      internal_collaborations: number;
    }> = [];
    const seenBlocked = new Set<string>();

    for (const blockedId of blockedIds) {
      if (seenBlocked.has(blockedId)) {
        continue;
      }

      const queue = [blockedId];
      const clusterIds: string[] = [];
      let internalCollaborations = 0;
      seenBlocked.add(blockedId);

      while (queue.length > 0) {
        const current = queue.shift()!;
        clusterIds.push(current);

        for (const collaboration of adjacency.get(current) ?? []) {
          const neighbor =
            collaboration.artistId1 === current
              ? collaboration.artistId2
              : collaboration.artistId1;

          if (!blockedIds.has(neighbor)) {
            continue;
          }

          internalCollaborations += collaboration.collaborationCount ?? 1;

          if (!seenBlocked.has(neighbor)) {
            seenBlocked.add(neighbor);
            queue.push(neighbor);
          }
        }
      }

      blockedClusters.push({
        cluster_id: `cluster:${clusterIds.sort().join(":")}`,
        artists: clusterIds
          .map((artistId) => artists.get(artistId))
          .filter(Boolean)
          .map((artist) => toGraphArtist(artist!, true)),
        internal_collaborations: Math.floor(internalCollaborations / 2),
      });
    }

    return {
      at_risk_artists: Array.from(atRiskMap.values()).sort(
        (left, right) => right.risk_score - left.risk_score,
      ),
      blocked_clusters: blockedClusters,
      summary: {
        total_blocked: blockedIds.size,
        total_at_risk: atRiskMap.size,
        avg_collaborations_per_blocked:
          blockedIds.size === 0
            ? 0
            : collaborations.length / blockedIds.size,
      },
    };
  },
});

export const health = query({
  args: {},
  handler: async (ctx) => {
    const [artists, collaborations, syncRuns] = await Promise.all([
      ctx.db.query("artists").collect(),
      ctx.db.query("artistCollaborations").collect(),
      ctx.db.query("platformSyncRuns").collect(),
    ]);

    const latestSync = syncRuns
      .map((run: Doc<"platformSyncRuns">) => run.completedAt ?? run.updatedAt ?? run.startedAt)
      .filter(Boolean)
      .sort()
      .at(-1);

    const syncLagSeconds = latestSync
      ? Math.max(0, Math.floor((Date.now() - new Date(latestSync).getTime()) / 1000))
      : 0;

    const status =
      !latestSync || syncLagSeconds > 60 * 60 * 24 * 7
        ? "unhealthy"
        : syncLagSeconds > 60 * 60 * 24
          ? "degraded"
          : "healthy";

    return {
      status,
      node_count: artists.length,
      edge_count: collaborations.length,
      last_sync: latestSync ?? new Date(0).toISOString(),
      sync_lag_seconds: syncLagSeconds,
    };
  },
});

export const stats = query({
  args: {},
  handler: async (ctx) => {
    const [artists, collaborations, tracks] = await Promise.all([
      ctx.db.query("artists").collect(),
      ctx.db.query("artistCollaborations").collect(),
      ctx.db.query("tracks").collect(),
    ]);

    return {
      artist_count: artists.length,
      collaboration_count: collaborations.length,
      label_count: 0,
      track_count: tracks.length,
    };
  },
});
