-- Seed Drake as showcase example with full discography and offenses
-- This provides a complete example of what the artist page can display

-- First, ensure the updated_at column exists on artists table
ALTER TABLE artists ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW();

-- Backfill existing rows
UPDATE artists SET updated_at = created_at WHERE updated_at IS NULL;

-- Get Drake's existing ID or use a well-known one
DO $$
DECLARE
    drake_id uuid;
BEGIN
    -- Get existing Drake ID or create one
    SELECT id INTO drake_id FROM artists WHERE canonical_name = 'Drake' LIMIT 1;

    IF drake_id IS NULL THEN
        drake_id := 'f50da45e-2e29-4954-a80a-8923c41d13d9'::uuid;
        INSERT INTO artists (id, canonical_name, external_ids, metadata, created_at, updated_at)
        VALUES (
            drake_id,
            'Drake',
            '{"spotify": "3TVXtAsR1Inumwj472S9r4", "apple_music": "271256", "musicbrainz": "b49b81cc-d5b7-4bdd-aadb-385df8de69a6"}',
            '{"image": "https://i.scdn.co/image/ab6761610000e5eb4293385d324db8558179afd9", "genres": ["Hip-Hop", "R&B", "Pop Rap", "Canadian Hip-Hop"], "popularity": 96}',
            NOW(),
            NOW()
        );
    END IF;

    -- Add Drake's discography (albums) - using ON CONFLICT to handle re-runs
    INSERT INTO albums (id, title, release_date, release_year, total_tracks, album_type, spotify_id, apple_music_id, created_at, updated_at)
    VALUES
        ('a1000001-0001-0000-0000-000000000001'::uuid, 'For All The Dogs', '2023-10-06', 2023, 23, 'album', '4czdORdCWP9umpbhFXK2fW', '1710304493', NOW(), NOW()),
        ('a1000001-0001-0000-0000-000000000002'::uuid, 'Her Loss', '2022-11-04', 2022, 16, 'album', '5DfSaCNpOz63oNIqP0hQvR', '1649413389', NOW(), NOW()),
        ('a1000001-0001-0000-0000-000000000003'::uuid, 'Honestly, Nevermind', '2022-06-17', 2022, 14, 'album', '3cf4iSSKd8ffTncbtKljXw', '1630019516', NOW(), NOW()),
        ('a1000001-0001-0000-0000-000000000004'::uuid, 'Certified Lover Boy', '2021-09-03', 2021, 21, 'album', '3SpBlxme9WbeQdI9kx7KAV', '1582068755', NOW(), NOW()),
        ('a1000001-0001-0000-0000-000000000005'::uuid, 'Dark Lane Demo Tapes', '2020-05-01', 2020, 14, 'compilation', '5l32yrJ5VGjOkrOODkbGhY', '1510842069', NOW(), NOW()),
        ('a1000001-0001-0000-0000-000000000006'::uuid, 'Scorpion', '2018-06-29', 2018, 25, 'album', '1ATL5GLyefJaxhQzSPVrLX', '1390828000', NOW(), NOW()),
        ('a1000001-0001-0000-0000-000000000007'::uuid, 'More Life', '2017-03-18', 2017, 22, 'playlist', '7Ix0FS4f1lK9lp90fJCR7z', '1211604108', NOW(), NOW()),
        ('a1000001-0001-0000-0000-000000000008'::uuid, 'Views', '2016-04-29', 2016, 20, 'album', '40GMAhriYJRO1rsY4YdrZb', '1097624362', NOW(), NOW()),
        ('a1000001-0001-0000-0000-000000000009'::uuid, 'If You''re Reading This It''s Too Late', '2015-02-13', 2015, 17, 'album', '0ptlfJfwGTy0Yvrk14JK1I', '966227040', NOW(), NOW()),
        ('a1000001-0001-0000-0000-00000000000a'::uuid, 'Nothing Was the Same', '2013-09-24', 2013, 15, 'album', '5OO3Sj75Qs8TkTu4VmZcXK', '696856923', NOW(), NOW()),
        ('a1000001-0001-0000-0000-00000000000b'::uuid, 'Take Care', '2011-11-15', 2011, 20, 'album', '6X1x82kppWZmDzlXXK3y3q', '480201780', NOW(), NOW()),
        ('a1000001-0001-0000-0000-00000000000c'::uuid, 'Thank Me Later', '2010-06-15', 2010, 14, 'album', '2AOsLyJKDWxJMVxuAZ5EBC', '375570249', NOW(), NOW())
    ON CONFLICT (id) DO UPDATE SET
        title = EXCLUDED.title,
        release_date = EXCLUDED.release_date,
        total_tracks = EXCLUDED.total_tracks,
        updated_at = NOW();

    -- Link albums to Drake via album_artists
    INSERT INTO album_artists (album_id, artist_id, is_primary, created_at)
    VALUES
        ('a1000001-0001-0000-0000-000000000001'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-000000000002'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-000000000003'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-000000000004'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-000000000005'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-000000000006'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-000000000007'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-000000000008'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-000000000009'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-00000000000a'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-00000000000b'::uuid, drake_id, true, NOW()),
        ('a1000001-0001-0000-0000-00000000000c'::uuid, drake_id, true, NOW())
    ON CONFLICT (album_id, artist_id) DO NOTHING;

    -- Add Drake's offenses (documented incidents)
    -- Using valid OffenseCategory values: sexual_misconduct, fraud, etc.
    INSERT INTO artist_offenses (id, artist_id, category, severity, title, description, incident_date, status, verification_status, created_at, updated_at)
    VALUES
        (
            'b1000001-0001-0000-0000-000000000001'::uuid,
            drake_id,
            'sexual_misconduct',
            'severe',
            'Inappropriate Relationship with Millie Bobby Brown',
            'Drake was revealed to have been texting then-14-year-old actress Millie Bobby Brown, giving her advice about "boys." The nature of a 31-year-old man privately texting a 14-year-old girl raised significant concerns.',
            '2018-09-01',
            'verified',
            'verified',
            NOW(),
            NOW()
        ),
        (
            'b1000001-0001-0000-0000-000000000002'::uuid,
            drake_id,
            'sexual_misconduct',
            'severe',
            'On-Stage Interaction with Underage Fan',
            'Video surfaced of Drake kissing and touching a 17-year-old fan on stage during a concert in Denver. After learning her age, he continued the interaction and made inappropriate comments.',
            '2010-05-16',
            'verified',
            'verified',
            NOW(),
            NOW()
        ),
        (
            'b1000001-0001-0000-0000-000000000003'::uuid,
            drake_id,
            'sexual_misconduct',
            'moderate',
            'Pattern of Dating Much Younger Women',
            'Drake has been documented dating or pursuing women significantly younger than him, including allegedly pursuing a relationship with model Bella Harris since she was 16.',
            '2018-10-01',
            'pending',
            'verified',
            NOW(),
            NOW()
        ),
        (
            'b1000001-0001-0000-0000-000000000004'::uuid,
            drake_id,
            'fraud',
            'moderate',
            'Ghostwriting and Authenticity Questions',
            'Reference tracks by Quentin Miller and others leaked, showing Drake uses ghostwriters while marketing himself as a skilled lyricist. This was highlighted in Pusha T''s diss track "The Story of Adidon."',
            '2015-07-22',
            'verified',
            'verified',
            NOW(),
            NOW()
        ),
        (
            'b1000001-0001-0000-0000-000000000005'::uuid,
            drake_id,
            'fraud',
            'minor',
            'Hidden Child Revelation',
            'Drake hid the existence of his son Adonis for over a year until exposed by Pusha T in "The Story of Adidon." Drake had been planning to reveal the child through an Adidas deal.',
            '2018-05-29',
            'verified',
            'verified',
            NOW(),
            NOW()
        )
    ON CONFLICT (id) DO UPDATE SET
        category = EXCLUDED.category,
        title = EXCLUDED.title,
        description = EXCLUDED.description,
        severity = EXCLUDED.severity,
        updated_at = NOW();

    -- Add evidence for the offenses
    -- Using 'url' (NOT NULL) column, credibility_score is integer 1-5
    INSERT INTO offense_evidence (id, offense_id, url, source_name, source_type, title, excerpt, published_date, credibility_score, is_primary_source, created_at)
    VALUES
        (
            'c1000001-0001-0000-0000-000000000001'::uuid,
            'b1000001-0001-0000-0000-000000000001'::uuid,
            'https://www.billboard.com/music/music-news/millie-bobby-brown-drake-friendship-emmys-8475631/',
            'Billboard',
            'news',
            'Millie Bobby Brown Discusses Drake Friendship at Emmys',
            'The Stranger Things star revealed that Drake texts her, saying "I love him...He''s honestly so fantastic...We just texted each other the other day and he was like, ''I miss you so much,'' and I was like, ''I miss you more.''"',
            '2018-09-18',
            4,
            false,
            NOW()
        ),
        (
            'c1000001-0001-0000-0000-000000000002'::uuid,
            'b1000001-0001-0000-0000-000000000002'::uuid,
            'https://www.complex.com/music/drake-kissing-underage-girl-on-stage-video',
            'Complex',
            'video',
            'Drake Kisses 17-Year-Old Fan on Stage',
            'Video footage from the concert shows Drake bringing a fan on stage, kissing her, and making comments about her body before learning she was 17.',
            '2010-05-16',
            5,
            false,
            NOW()
        ),
        (
            'c1000001-0001-0000-0000-000000000003'::uuid,
            'b1000001-0001-0000-0000-000000000004'::uuid,
            'https://www.thefader.com/2015/07/22/drake-quentin-miller-reference-tracks',
            'The Fader',
            'news',
            'Reference Tracks from Quentin Miller Leak',
            'Multiple reference tracks by Quentin Miller for Drake songs including "10 Bands" and "Know Yourself" were leaked, revealing the extent of collaboration.',
            '2015-07-22',
            4,
            false,
            NOW()
        ),
        (
            'c1000001-0001-0000-0000-000000000004'::uuid,
            'b1000001-0001-0000-0000-000000000005'::uuid,
            'https://genius.com/Pusha-t-the-story-of-adidon-lyrics',
            'Genius',
            'primary_source',
            'Pusha T - The Story of Adidon',
            '"You are hiding a child, let that boy come home / Deadbeat motherfucka playin'' border patrol"',
            '2018-05-29',
            5,
            true,
            NOW()
        )
    ON CONFLICT (id) DO UPDATE SET
        title = EXCLUDED.title,
        excerpt = EXCLUDED.excerpt;

END $$;
