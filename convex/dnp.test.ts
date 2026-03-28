/// <reference types="vite/client" />
import { convexTest } from "convex-test";
import { expect, test, describe } from "vitest";
import { api } from "./_generated/api";
import schema from "./schema";

const modules = import.meta.glob("./**/*.ts");

const TEST_IDENTITY = {
  tokenIdentifier: "test|user_001",
  email: "drake-blocker@test.com",
  name: "Test Blocker",
};

function setupUserAndArtists(t: ReturnType<typeof convexTest>) {
  return t.run(async (ctx) => {
    const now = new Date().toISOString();

    // Create test user
    const userId = await ctx.db.insert("users", {
      legacyKey: "test:user:001",
      authSubject: TEST_IDENTITY.tokenIdentifier,
      email: TEST_IDENTITY.email,
      displayName: "Test Blocker",
      createdAt: now,
      updatedAt: now,
    });

    // Create test artists
    const drakeId = await ctx.db.insert("artists", {
      legacyKey: "test:artist:drake",
      canonicalName: "Drake",
      externalIds: { spotify: "3TVXtAsR1Inumwj472S9r4" },
      metadata: { genres: ["hip-hop", "rap"] },
      aliases: ["Drizzy", "Champagne Papi"],
      status: "active",
      createdAt: now,
      updatedAt: now,
    });

    const weekndId = await ctx.db.insert("artists", {
      legacyKey: "test:artist:weeknd",
      canonicalName: "The Weeknd",
      externalIds: { spotify: "1Xyo4u8uXC1ZmMpatF05PJ" },
      metadata: { genres: ["r-and-b", "pop"] },
      aliases: [],
      status: "active",
      createdAt: now,
      updatedAt: now,
    });

    const cleanArtistId = await ctx.db.insert("artists", {
      legacyKey: "test:artist:clean",
      canonicalName: "Clean Artist",
      externalIds: { spotify: "clean123" },
      metadata: {},
      aliases: [],
      status: "active",
      createdAt: now,
      updatedAt: now,
    });

    return { userId, drakeId, weekndId, cleanArtistId };
  });
}

describe("dnp:listCurrentUser", () => {
  test("returns empty list for user with no blocks", async () => {
    const t = convexTest(schema, modules);
    await setupUserAndArtists(t);

    const result = await t.withIdentity(TEST_IDENTITY).query(api.dnp.listCurrentUser, {});
    expect(result).toHaveLength(0);
  });

  test("returns blocked artists", async () => {
    const t = convexTest(schema, modules);
    const { userId, drakeId } = await setupUserAndArtists(t);

    // Add a block
    await t.run(async (ctx) => {
      await ctx.db.insert("userArtistBlocks", {
        legacyKey: `test:block:${userId}:${drakeId}`,
        userId,
        artistId: drakeId,
        tags: ["violence"],
        note: "Test block",
        source: "test",
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      });
    });

    const result = await t.withIdentity(TEST_IDENTITY).query(api.dnp.listCurrentUser, {});
    expect(result).toHaveLength(1);
    expect(result[0]?.artist.canonical_name).toBe("Drake");
    expect(result[0]?.tags).toEqual(["violence"]);
  });
});

describe("dnp:searchArtists", () => {
  test("finds artists by name", async () => {
    const t = convexTest(schema, modules);
    await setupUserAndArtists(t);

    const result = await t.withIdentity(TEST_IDENTITY).query(api.dnp.searchArtists, {
      query: "Drake",
    });

    expect(result.artists.length).toBeGreaterThan(0);
    expect(result.artists[0].canonical_name).toBe("Drake");
    expect(result.artists[0].source).toBe("convex");
  });

  test("returns empty for non-matching query", async () => {
    const t = convexTest(schema, modules);
    await setupUserAndArtists(t);

    const result = await t.withIdentity(TEST_IDENTITY).query(api.dnp.searchArtists, {
      query: "zzz_nonexistent_artist_zzz",
    });

    expect(result.artists).toHaveLength(0);
  });
});

describe("dnp:addArtistBlock", () => {
  test("adds block by artist ID", async () => {
    const t = convexTest(schema, modules);
    const { drakeId } = await setupUserAndArtists(t);

    const result = await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.addArtistBlock, {
      artistId: drakeId,
      tags: ["test-tag"],
      note: "Blocked for testing",
    });

    expect(result).not.toBeNull();
    expect(result?.artist.canonical_name).toBe("Drake");
    expect(result?.tags).toEqual(["test-tag"]);
  });

  test("adds block by query string", async () => {
    const t = convexTest(schema, modules);
    await setupUserAndArtists(t);

    const result = await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.addArtistBlock, {
      query: "The Weeknd",
      tags: [],
    });

    expect(result).not.toBeNull();
    expect(result?.artist.canonical_name).toBe("The Weeknd");
  });

  test("updates existing block instead of duplicating", async () => {
    const t = convexTest(schema, modules);
    const { drakeId } = await setupUserAndArtists(t);

    await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.addArtistBlock, {
      artistId: drakeId,
      tags: ["original"],
    });

    await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.addArtistBlock, {
      artistId: drakeId,
      tags: ["updated"],
      note: "Updated note",
    });

    const list = await t.withIdentity(TEST_IDENTITY).query(api.dnp.listCurrentUser, {});
    expect(list).toHaveLength(1);
    expect(list[0]?.tags).toEqual(["updated"]);
  });
});

describe("dnp:removeArtistBlock", () => {
  test("removes existing block", async () => {
    const t = convexTest(schema, modules);
    const { drakeId } = await setupUserAndArtists(t);

    await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.addArtistBlock, {
      artistId: drakeId,
    });

    const result = await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.removeArtistBlock, {
      artistId: drakeId,
    });

    expect(result.success).toBe(true);

    const list = await t.withIdentity(TEST_IDENTITY).query(api.dnp.listCurrentUser, {});
    expect(list).toHaveLength(0);
  });

  test("succeeds even when no block exists", async () => {
    const t = convexTest(schema, modules);
    const { drakeId } = await setupUserAndArtists(t);

    const result = await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.removeArtistBlock, {
      artistId: drakeId,
    });

    expect(result.success).toBe(true);
  });
});

describe("dnp:importBlocklist", () => {
  test("imports multiple artists", async () => {
    const t = convexTest(schema, modules);
    await setupUserAndArtists(t);

    const result = await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.importBlocklist, {
      entries: [
        { artistName: "Drake", tags: ["import"] },
        { artistName: "The Weeknd" },
        { artistName: "Unknown Artist That Does Not Exist" },
      ],
    });

    expect(result.imported).toBe(2);
    expect(result.skipped).toBe(1);
    expect(result.total).toBe(3);
  });

  test("skips already-blocked artists", async () => {
    const t = convexTest(schema, modules);
    const { drakeId } = await setupUserAndArtists(t);

    // Pre-block Drake
    await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.addArtistBlock, {
      artistId: drakeId,
    });

    const result = await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.importBlocklist, {
      entries: [{ artistName: "Drake" }],
    });

    expect(result.imported).toBe(0);
    expect(result.skipped).toBe(1);
  });
});

describe("dnp:exportBlocklist", () => {
  test("exports all user blocks", async () => {
    const t = convexTest(schema, modules);
    const { drakeId, weekndId } = await setupUserAndArtists(t);

    await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.addArtistBlock, {
      artistId: drakeId,
      tags: ["violence"],
    });
    await t.withIdentity(TEST_IDENTITY).mutation(api.dnp.addArtistBlock, {
      artistId: weekndId,
    });

    const result = await t.withIdentity(TEST_IDENTITY).query(api.dnp.exportBlocklist, {});
    expect(result.total).toBe(2);
    expect(result.entries).toHaveLength(2);
    expect(result.exported_at).toBeDefined();

    const names = result.entries.map((e: any) => e.artist_name);
    expect(names).toContain("Drake");
    expect(names).toContain("The Weeknd");
  });
});

describe("dnp:_upsertSpotifyArtist", () => {
  test("creates new artist from Spotify data", async () => {
    const t = convexTest(schema, modules);

    const artistId = await t.mutation(api.dnp._upsertSpotifyArtist, {
      spotifyId: "new_spotify_id",
      name: "New Spotify Artist",
      genres: ["pop"],
      imageUrl: "https://example.com/image.jpg",
    });

    expect(artistId).toBeDefined();

    const artist = await t.run(async (ctx) => {
      return await ctx.db.get(artistId);
    });

    expect(artist?.canonicalName).toBe("New Spotify Artist");
    expect((artist?.externalIds as any)?.spotify).toBe("new_spotify_id");
  });

  test("returns existing artist if name matches", async () => {
    const t = convexTest(schema, modules);
    const { drakeId } = await setupUserAndArtists(t);

    const resultId = await t.mutation(api.dnp._upsertSpotifyArtist, {
      spotifyId: "3TVXtAsR1Inumwj472S9r4",
      name: "Drake",
      genres: ["hip-hop"],
    });

    expect(String(resultId)).toBe(String(drakeId));
  });
});
