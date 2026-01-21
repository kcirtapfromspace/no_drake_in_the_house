-- Migration: 030_seed_artists_batch3_hate.sql
-- Batch 3: Hate Speech, Racism, Antisemitism, and Homophobia Artists (~80 artists)

-- =============================================
-- PART 1: INSERT NEW ARTISTS - Hate Speech Categories
-- =============================================

-- Racism/White Supremacy
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Skrewdriver', '{}'::jsonb, '{"genres": ["punk rock", "rock against communism"], "note": "Neo-Nazi band", "popularity": 20}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Skrewdriver');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Blue Eyed Devils', '{}'::jsonb, '{"genres": ["white power", "rock"], "note": "White supremacist band", "popularity": 10}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Blue Eyed Devils');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Prussian Blue', '{}'::jsonb, '{"genres": ["folk", "pop"], "note": "Former white nationalist duo", "popularity": 15}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Prussian Blue');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Saga', '{}'::jsonb, '{"genres": ["rock"], "note": "White power singer", "popularity": 10}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Saga');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Johnny Rebel', '{}'::jsonb, '{"genres": ["country"], "note": "Racist novelty songs", "popularity": 15}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Johnny Rebel');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'David Allan Coe', '{"spotify": "27R7YpJlv6xm7HXNgKp5ER"}'::jsonb, '{"genres": ["country", "outlaw country"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'David Allan Coe');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Charlie Daniels', '{"spotify": "0LHlK1kD0x7xNlJ6ezYzPB"}'::jsonb, '{"genres": ["country", "southern rock"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Charlie Daniels');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Hank Williams Jr.', '{"spotify": "1K4x4nRUBGOaJtV2RJjfqV"}'::jsonb, '{"genres": ["country", "southern rock"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Hank Williams Jr.');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Travis Tritt', '{"spotify": "6n41NIsNqvUcyGhRo91MOG"}'::jsonb, '{"genres": ["country"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Travis Tritt');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Aaron Lewis', '{"spotify": "7Jjl9Vl6GSPFaAHCGfkbcS"}'::jsonb, '{"genres": ["rock", "country"], "note": "Staind vocalist", "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Aaron Lewis');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'John Mayer', '{"spotify": "0hEurMDQu99nJRq8pTxO14"}'::jsonb, '{"genres": ["pop", "rock", "blues"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'John Mayer');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Elvis Costello', '{"spotify": "2bgTY4UwhfBYhGT4HUYStN"}'::jsonb, '{"genres": ["rock", "new wave"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Elvis Costello');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Mark Kozelek', '{"spotify": "1a0Xp5MWkkluLhTQnPOXHi"}'::jsonb, '{"genres": ["indie rock", "folk"], "note": "Sun Kil Moon", "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Mark Kozelek');

-- Homophobia - More Dancehall Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bounty Killer', '{"spotify": "7xwcMJzgFHU5BUVWP0cA2b"}'::jsonb, '{"genres": ["dancehall", "reggae"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bounty Killer');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Capleton', '{"spotify": "18EPTqMSXjIEjQJmWXpIOS"}'::jsonb, '{"genres": ["dancehall", "reggae"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Capleton');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'TOK', '{"spotify": "5Nc5K7TKRmXXgqFBYCBJmk"}'::jsonb, '{"genres": ["dancehall", "reggae"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'TOK');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Shabba Ranks', '{"spotify": "2MImlcVuUKLOIhXfF8Bm5O"}'::jsonb, '{"genres": ["dancehall", "reggae"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Shabba Ranks');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Alkaline', '{"spotify": "4sS0Jc6SjFJH5cxIDJdj6b"}'::jsonb, '{"genres": ["dancehall"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Alkaline');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Vybz Kartel', '{"spotify": "7yJDXdNJPcIe3a2Qp6yATG"}'::jsonb, '{"genres": ["dancehall"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Vybz Kartel');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'T.O.K.', '{"spotify": "5Nc5K7TKRmXXgqFBYCBJmk"}'::jsonb, '{"genres": ["dancehall", "reggae"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'T.O.K.');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Mavado', '{"spotify": "5VGaT0HZX9WbYpFzJZ3eB6"}'::jsonb, '{"genres": ["dancehall"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Mavado');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Popcaan', '{"spotify": "3qnGvpP8Yth1AqSBMqON5x"}'::jsonb, '{"genres": ["dancehall"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Popcaan');

-- Hip Hop Artists with Homophobic History
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT '50 Cent', '{"spotify": "3q7HBObVc0L8jNeTe5Gofh"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = '50 Cent');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tyler, The Creator', '{"spotify": "4V8LLVI7PbaPR0K2TGSxFF"}'::jsonb, '{"genres": ["hip hop", "alternative hip hop"], "popularity": 88}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tyler, The Creator');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Wayne', '{"spotify": "55Aa2cqylxrFIXC767Z865"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 88}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Wayne');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Migos', '{"spotify": "6oMuImdp5ZcFhWP0ESe6mG"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Migos');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Boosie Badazz', '{"spotify": "36E7oYfz3LLRto6yCpf85B"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Boosie Badazz');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Trick Daddy', '{"spotify": "6R7i41HHUBiWHzBqAn7fNs"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Trick Daddy');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jadakiss', '{"spotify": "1wJZLPeGPkLPVRqpqxBs3b"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jadakiss');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Brand Nubian', '{"spotify": "1x7RQWPJ1TKH1VOMdWmHrR"}'::jsonb, '{"genres": ["hip hop"], "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Brand Nubian');

-- Antisemitism Cases
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Public Enemy', '{"spotify": "6Mo9PoU6svvhgEum7wh2Nd"}'::jsonb, '{"genres": ["hip hop", "political hip hop"], "popularity": 62}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Public Enemy');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Professor Griff', '{}'::jsonb, '{"genres": ["hip hop"], "note": "Former Public Enemy member", "popularity": 25}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Professor Griff');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Nick Cannon', '{"spotify": "3PzpoU3j6J6BDLEH1hZP9h"}'::jsonb, '{"genres": ["hip hop", "r&b"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Nick Cannon');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Wiley', '{"spotify": "3B0G3wVyhbPPqOyg5a3IQ1"}'::jsonb, '{"genres": ["grime", "uk hip hop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Wiley');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Talib Kweli', '{"spotify": "0WCXqWYqPHSNFz9UD5a6ch"}'::jsonb, '{"genres": ["hip hop", "conscious hip hop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Talib Kweli');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'GZA', '{"spotify": "3GACrzn9EJU4hFvqr1hRTa"}'::jsonb, '{"genres": ["hip hop"], "note": "Wu-Tang Clan member", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'GZA');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jay Electronica', '{"spotify": "0Y4inQKMFBelpkmDgKFVAU"}'::jsonb, '{"genres": ["hip hop"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jay Electronica');

-- Black Metal / NSBM (National Socialist Black Metal)
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Graveland', '{"spotify": "2rKDCl8sGNpqvDxXlQBV3A"}'::jsonb, '{"genres": ["black metal"], "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Graveland');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Absurd', '{}'::jsonb, '{"genres": ["black metal", "NSBM"], "popularity": 20}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Absurd');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Nokturnal Mortum', '{"spotify": "4DQYpN6yZqVhxIYn3PXKkZ"}'::jsonb, '{"genres": ["black metal", "folk metal"], "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Nokturnal Mortum');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Peste Noire', '{}'::jsonb, '{"genres": ["black metal"], "popularity": 30}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Peste Noire');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Drudkh', '{"spotify": "5K0u4vJr3CZr2t8Wv8BQKV"}'::jsonb, '{"genres": ["black metal", "folk metal"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Drudkh');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'M8l8th', '{}'::jsonb, '{"genres": ["black metal", "NSBM"], "popularity": 15}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'M8l8th');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Goatmoon', '{}'::jsonb, '{"genres": ["black metal"], "popularity": 25}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Goatmoon');

-- Rock Artists with Controversial Statements
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Phil Anselmo', '{"spotify": "69bj0StGfCdX8NMGNE3vIG"}'::jsonb, '{"genres": ["metal", "groove metal"], "note": "Pantera vocalist", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Phil Anselmo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Pantera', '{"spotify": "58dXNQoKlr48rBqLIFsGvB"}'::jsonb, '{"genres": ["metal", "groove metal"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Pantera');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jon Schaffer', '{}'::jsonb, '{"genres": ["power metal"], "note": "Iced Earth guitarist", "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jon Schaffer');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Iced Earth', '{"spotify": "6uRJKLTPw6HcfJWW3FPDd7"}'::jsonb, '{"genres": ["power metal", "thrash metal"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Iced Earth');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Michale Graves', '{"spotify": "22Qk9z9m1RZMjF8S9zDjhC"}'::jsonb, '{"genres": ["punk rock", "horror punk"], "note": "Former Misfits vocalist", "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Michale Graves');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Killer Mike', '{"spotify": "5m8H6zSadhu1j9Yi04VLqD"}'::jsonb, '{"genres": ["hip hop", "southern hip hop"], "popularity": 68}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Killer Mike');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Scarface', '{"spotify": "6Yp3gBM8bLlCHpxl6XHFjZ"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Scarface');

-- =============================================
-- PART 2: INSERT OFFENSES FOR NEW ARTISTS
-- =============================================

-- Skrewdriver
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'hate_speech'::offense_category, 'egregious'::offense_severity,
       'White Supremacist Band',
       'Band became openly neo-Nazi in 1982. Frontman Ian Stuart Donaldson founded Blood & Honour white power music network. Produced explicitly racist music until his death in 1993.',
       '1982-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Skrewdriver'
ON CONFLICT DO NOTHING;

-- David Allan Coe
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'racism'::offense_category, 'severe'::offense_severity,
       'Racist Album Recordings',
       'Recorded extremely racist underground albums in the 1980s. Songs contained racial slurs and white supremacist content. Later claimed they were recorded under duress.',
       '1982-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'David Allan Coe'
ON CONFLICT DO NOTHING;

-- Hank Williams Jr.
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'hate_speech'::offense_category, 'moderate'::offense_severity,
       'Obama-Hitler Comparison',
       'Compared President Obama to Hitler on Fox News, resulting in ESPN dropping his Monday Night Football theme song. Made numerous other controversial political statements.',
       '2011-10-03', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Hank Williams Jr.'
ON CONFLICT DO NOTHING;

-- John Mayer
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'racism'::offense_category, 'moderate'::offense_severity,
       'Playboy Interview Racial Slur',
       'Used N-word in Playboy interview when discussing his dating preferences. Also made comments about his genitalia being a "white supremacist." Later apologized.',
       '2010-02-10', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'John Mayer'
ON CONFLICT DO NOTHING;

-- Elvis Costello
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'racism'::offense_category, 'moderate'::offense_severity,
       'Racial Slurs About Ray Charles and James Brown',
       'During 1979 drunken argument with Bonnie Bramlett, used racial slurs about Ray Charles and James Brown. Incident nearly ended his US career. Later apologized extensively.',
       '1979-03-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Elvis Costello'
ON CONFLICT DO NOTHING;

-- Phil Anselmo
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'racism'::offense_category, 'severe'::offense_severity,
       'Nazi Salute and White Power Shout',
       'Gave Nazi salute and shouted "white power" on stage at Dimebash tribute concert. Also used racial slurs in past interviews. Apologized claiming it was a joke about white wine.',
       '2016-01-22', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Phil Anselmo'
ON CONFLICT DO NOTHING;

-- Bounty Killer
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'severe'::offense_severity,
       'Murder Music Lyrics',
       'Multiple songs calling for violence against LGBTQ+ people. Part of the "murder music" controversy. Faced concert cancellations and protests worldwide.',
       '2004-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Bounty Killer'
ON CONFLICT DO NOTHING;

-- Capleton
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'severe'::offense_severity,
       'Anti-LGBTQ Lyrics and Statements',
       'Numerous songs promoting violence against gay people. "Burn dem" lyrics explicitly call for murder. Concerts cancelled worldwide due to protests.',
       '2004-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Capleton'
ON CONFLICT DO NOTHING;

-- Vybz Kartel
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'moderate'::offense_severity,
       'Homophobic Lyrics',
       'Multiple songs with anti-gay lyrics. Continued pattern in dancehall of homophobic content.',
       '2010-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Vybz Kartel'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder Conviction',
       'Convicted of murder of Clive "Lizard" Williams. Sentenced to life in prison with minimum 35 years. Conviction upheld on appeal.',
       '2011-08-16', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Vybz Kartel'
ON CONFLICT DO NOTHING;

-- 50 Cent
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'moderate'::offense_severity,
       'Homophobic Lyrics and Comments',
       'Numerous songs with homophobic slurs. Made public homophobic comments about other artists. Has been criticized by LGBTQ+ advocacy groups.',
       '2005-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = '50 Cent'
ON CONFLICT DO NOTHING;

-- Tyler, The Creator
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'moderate'::offense_severity,
       'Early Homophobic Lyrics',
       'Early albums contained extensive use of homophobic slurs. Banned from UK for content. Later came out as LGBTQ+ and has expressed regret for past lyrics.',
       '2011-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tyler, The Creator'
ON CONFLICT DO NOTHING;

-- Boosie Badazz
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'severe'::offense_severity,
       'Repeated Homophobic Statements',
       'Made numerous public homophobic statements including saying he would disown gay children. Attacked Lil Nas X and Dwyane Wade''s daughter. Banned from Planet Fitness.',
       '2020-03-05', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Boosie Badazz'
ON CONFLICT DO NOTHING;

-- Nick Cannon
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'antisemitism'::offense_category, 'moderate'::offense_severity,
       'Antisemitic Podcast Comments',
       'Made antisemitic comments on podcast promoting conspiracy theories about Jewish people controlling media and banking. Fired by ViacomCBS. Later apologized after education.',
       '2020-07-14', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Nick Cannon'
ON CONFLICT DO NOTHING;

-- Wiley
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'antisemitism'::offense_category, 'severe'::offense_severity,
       'Antisemitic Twitter Tirade',
       'Posted antisemitic tweets over 48 hours comparing Jews to KKK, promoting conspiracy theories. Permanently banned from Twitter, Facebook, Instagram. Lost management and label.',
       '2020-07-24', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Wiley'
ON CONFLICT DO NOTHING;

-- Professor Griff
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'antisemitism'::offense_category, 'severe'::offense_severity,
       'Antisemitic Interview Comments',
       'Stated Jews were responsible for "majority of wickedness that goes on across the globe." Dismissed from Public Enemy. Continued making antisemitic statements.',
       '1989-05-22', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Professor Griff'
ON CONFLICT DO NOTHING;

-- Jay Electronica
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'antisemitism'::offense_category, 'moderate'::offense_severity,
       'Nation of Islam Antisemitic Content',
       'Album with Jay-Z contained antisemitic Nation of Islam content. Praised Louis Farrakhan. Tweeted conspiracy theories about Jewish people.',
       '2020-03-13', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jay Electronica'
ON CONFLICT DO NOTHING;

-- Jon Schaffer
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Capitol Riot Participation',
       'Participated in January 6th Capitol riot. Pleaded guilty to obstruction of Congress and entering Capitol with deadly weapon. Cooperated with investigation.',
       '2021-01-06', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jon Schaffer'
ON CONFLICT DO NOTHING;

-- Graveland
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'hate_speech'::offense_category, 'severe'::offense_severity,
       'NSBM and White Supremacy',
       'Associated with National Socialist Black Metal scene. Album artwork and lyrics promote white supremacist themes. Concert cancellations due to Nazi associations.',
       '1994-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Graveland'
ON CONFLICT DO NOTHING;

-- Absurd
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder and Nazi Ideology',
       'Three founding members murdered classmate Sandro Beyer in 1993. Became openly Nazi band in prison. Continue as NSBM band.',
       '1993-04-29', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Absurd'
ON CONFLICT DO NOTHING;

-- Migos
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'moderate'::offense_severity,
       'Homophobic Comments About iLoveMakonnen',
       'Made disparaging comments about iLoveMakonnen coming out as gay. Suggested it hurt his career. Later expressed regret.',
       '2017-02-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Migos'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 3: INSERT EVIDENCE FOR NEW OFFENSES
-- =============================================

-- Phil Anselmo evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/pantera-singer-apologizes-for-white-power-nazi-salute-video-81816/',
       'Rolling Stone', 'news',
       'Pantera Singer Apologizes for ''White Power'' Salute',
       '2016-01-28', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Phil Anselmo' AND ao.title = 'Nazi Salute and White Power Shout'
ON CONFLICT DO NOTHING;

-- Nick Cannon evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2020/07/15/arts/television/nick-cannon-fired-viacomcbs.html',
       'New York Times', 'news',
       'ViacomCBS Fires Nick Cannon Over Anti-Semitic Comments',
       '2020-07-15', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Nick Cannon' AND ao.title = 'Antisemitic Podcast Comments'
ON CONFLICT DO NOTHING;

-- Wiley evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.theguardian.com/music/2020/jul/27/wiley-dropped-by-management-over-antisemitic-posts',
       'The Guardian', 'news',
       'Wiley Dropped By Management Over Antisemitic Posts',
       '2020-07-27', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Wiley' AND ao.title = 'Antisemitic Twitter Tirade'
ON CONFLICT DO NOTHING;

-- Vybz Kartel evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.bbc.com/news/world-latin-america-26781584',
       'BBC News', 'news',
       'Vybz Kartel: Jamaican dancehall star guilty of murder',
       '2014-03-13', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Vybz Kartel' AND ao.title = 'Murder Conviction'
ON CONFLICT DO NOTHING;

-- Jon Schaffer evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.justice.gov/usao-dc/defendants/schaffer-jon-ryan',
       'US Department of Justice', 'court',
       'Jon Ryan Schaffer Capitol Riot Case',
       '2021-04-16', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Jon Schaffer' AND ao.title = 'Capitol Riot Participation'
ON CONFLICT DO NOTHING;

-- John Mayer evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/john-mayer-issues-apology-for-racial-slur-in-playboy-interview-102671/',
       'Rolling Stone', 'news',
       'John Mayer Issues Apology for Racial Slur in Playboy Interview',
       '2010-02-10', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'John Mayer' AND ao.title = 'Playboy Interview Racial Slur'
ON CONFLICT DO NOTHING;

-- Hank Williams Jr. evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2011/10/07/sports/football/espn-pulls-hank-williams-jr-s-song-after-his-comments.html',
       'New York Times', 'news',
       'ESPN Pulls Hank Williams Jr.''s Song After His Comments',
       '2011-10-07', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Hank Williams Jr.' AND ao.title = 'Obama-Hitler Comparison'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 4: INSERT PLATFORM IDS
-- =============================================

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '27R7YpJlv6xm7HXNgKp5ER', 'verified', 1.0
FROM artists WHERE canonical_name = 'David Allan Coe'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1K4x4nRUBGOaJtV2RJjfqV', 'verified', 1.0
FROM artists WHERE canonical_name = 'Hank Williams Jr.'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0hEurMDQu99nJRq8pTxO14', 'verified', 1.0
FROM artists WHERE canonical_name = 'John Mayer'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2bgTY4UwhfBYhGT4HUYStN', 'verified', 1.0
FROM artists WHERE canonical_name = 'Elvis Costello'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7xwcMJzgFHU5BUVWP0cA2b', 'verified', 1.0
FROM artists WHERE canonical_name = 'Bounty Killer'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '18EPTqMSXjIEjQJmWXpIOS', 'verified', 1.0
FROM artists WHERE canonical_name = 'Capleton'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7yJDXdNJPcIe3a2Qp6yATG', 'verified', 1.0
FROM artists WHERE canonical_name = 'Vybz Kartel'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3q7HBObVc0L8jNeTe5Gofh', 'verified', 1.0
FROM artists WHERE canonical_name = '50 Cent'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4V8LLVI7PbaPR0K2TGSxFF', 'verified', 1.0
FROM artists WHERE canonical_name = 'Tyler, The Creator'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6oMuImdp5ZcFhWP0ESe6mG', 'verified', 1.0
FROM artists WHERE canonical_name = 'Migos'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3PzpoU3j6J6BDLEH1hZP9h', 'verified', 1.0
FROM artists WHERE canonical_name = 'Nick Cannon'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3B0G3wVyhbPPqOyg5a3IQ1', 'verified', 1.0
FROM artists WHERE canonical_name = 'Wiley'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0Y4inQKMFBelpkmDgKFVAU', 'verified', 1.0
FROM artists WHERE canonical_name = 'Jay Electronica'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '69bj0StGfCdX8NMGNE3vIG', 'verified', 1.0
FROM artists WHERE canonical_name = 'Phil Anselmo'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '58dXNQoKlr48rBqLIFsGvB', 'verified', 1.0
FROM artists WHERE canonical_name = 'Pantera'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6uRJKLTPw6HcfJWW3FPDd7', 'verified', 1.0
FROM artists WHERE canonical_name = 'Iced Earth'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6Mo9PoU6svvhgEum7wh2Nd', 'verified', 1.0
FROM artists WHERE canonical_name = 'Public Enemy'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5VGaT0HZX9WbYpFzJZ3eB6', 'verified', 1.0
FROM artists WHERE canonical_name = 'Mavado'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3qnGvpP8Yth1AqSBMqON5x', 'verified', 1.0
FROM artists WHERE canonical_name = 'Popcaan'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2MImlcVuUKLOIhXfF8Bm5O', 'verified', 1.0
FROM artists WHERE canonical_name = 'Shabba Ranks'
ON CONFLICT (artist_id, platform) DO NOTHING;
