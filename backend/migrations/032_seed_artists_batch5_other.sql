-- Migration: 032_seed_artists_batch5_other.sql
-- Batch 5: Additional artists covering remaining categories and international artists

-- =============================================
-- PART 1: INSERT NEW ARTISTS - Mixed Categories
-- =============================================

-- Latin Artists with Controversies
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bad Bunny', '{"spotify": "4q3ewBCX7sLwd24euuV69X"}'::jsonb, '{"genres": ["reggaeton", "latin trap"], "popularity": 95}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bad Bunny');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'J Balvin', '{"spotify": "1vyhD5VmyZ7KMfW5gqLgo5"}'::jsonb, '{"genres": ["reggaeton", "latin pop"], "popularity": 90}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'J Balvin');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Daddy Yankee', '{"spotify": "4VMYDCV2IEDYJArk749S6m"}'::jsonb, '{"genres": ["reggaeton", "latin hip hop"], "popularity": 85}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Daddy Yankee');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Don Omar', '{"spotify": "0eDvMgVFoNV3TpwtrVCoTj"}'::jsonb, '{"genres": ["reggaeton"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Don Omar');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Arcangel', '{"spotify": "0OdUWJ0sBjDrqHygGUXeCF"}'::jsonb, '{"genres": ["reggaeton", "latin trap"], "popularity": 78}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Arcangel');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Farruko', '{"spotify": "329e4yvIujISKGKz1BZZbO"}'::jsonb, '{"genres": ["reggaeton", "latin trap"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Farruko');

-- K-Pop Industry Issues
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Seungri', '{}'::jsonb, '{"genres": ["k-pop"], "note": "Former Big Bang member", "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Seungri');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jung Joon-young', '{}'::jsonb, '{"genres": ["k-pop", "rock"], "popularity": 30}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jung Joon-young');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Kris Wu', '{"spotify": "5vJFZJbGHhH7aXLqQ9dG3Q"}'::jsonb, '{"genres": ["hip hop", "r&b"], "note": "Former EXO member", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Kris Wu');

-- British Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Dappy', '{"spotify": "4E1BVt1FLVvN9kV7aSrSoa"}'::jsonb, '{"genres": ["hip hop", "grime"], "note": "N-Dubz member", "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Dappy');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'N-Dubz', '{"spotify": "69Y9KHnELnVjJGfkA7OPnQ"}'::jsonb, '{"genres": ["hip hop", "grime"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'N-Dubz');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Professor Green', '{"spotify": "5bTjWn2DjDjXCsf5N0e8wS"}'::jsonb, '{"genres": ["hip hop", "grime"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Professor Green');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Giggs', '{"spotify": "5Y5TRrQiqgUO4S36tzjIRZ"}'::jsonb, '{"genres": ["hip hop", "grime"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Giggs');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Skepta', '{"spotify": "2p1fiYHYiXz9qi0JJyxBzN"}'::jsonb, '{"genres": ["grime", "hip hop"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Skepta');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Stormzy', '{"spotify": "2SrSdSvpminqmStGELCSNd"}'::jsonb, '{"genres": ["grime", "hip hop"], "popularity": 78}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Stormzy');

-- Classic Rock Controversies
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Gene Simmons', '{"spotify": "2RV7GRTIsTh8vMRjE3pDgC"}'::jsonb, '{"genres": ["rock", "hard rock"], "note": "KISS bassist", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Gene Simmons');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'KISS', '{"spotify": "07XSN3sPlIlB2L2XNcTwJw"}'::jsonb, '{"genres": ["rock", "hard rock"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'KISS');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Steven Adler', '{}'::jsonb, '{"genres": ["rock", "hard rock"], "note": "Former Guns N'' Roses drummer", "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Steven Adler');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Scott Weiland', '{"spotify": "2Cv9UGXpmIzZ7kkTbzk3HH"}'::jsonb, '{"genres": ["rock", "grunge"], "note": "Stone Temple Pilots vocalist", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Scott Weiland');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Layne Staley', '{}'::jsonb, '{"genres": ["rock", "grunge"], "note": "Alice in Chains vocalist", "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Layne Staley');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Kurt Cobain', '{"spotify": "0ByKFOYGfK6KM4TdEvqYEt"}'::jsonb, '{"genres": ["rock", "grunge"], "note": "Nirvana vocalist", "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Kurt Cobain');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Alice in Chains', '{"spotify": "64tNsm6TnZe2zpcMVMOoHL"}'::jsonb, '{"genres": ["rock", "grunge", "metal"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Alice in Chains');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Stone Temple Pilots', '{"spotify": "2UazAtjfzqBF0Nho2awK4z"}'::jsonb, '{"genres": ["rock", "grunge"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Stone Temple Pilots');

-- EDM/DJ Controversies
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'DJ Khaled', '{"spotify": "0QHgL1lAIqAw0HtD7YldmP"}'::jsonb, '{"genres": ["hip hop", "r&b"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'DJ Khaled');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Marshmello', '{"spotify": "64KEffDW9EtZ1y2vBYgq8T"}'::jsonb, '{"genres": ["electronic", "EDM"], "popularity": 85}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Marshmello');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bassnectar', '{"spotify": "73YYnb9vLiWZ5U1EGZj65v"}'::jsonb, '{"genres": ["electronic", "dubstep"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bassnectar');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Space Jesus', '{"spotify": "0hAQNlZFOgfqJqhMUy5mQT"}'::jsonb, '{"genres": ["electronic", "dubstep"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Space Jesus');

-- Country Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Toby Keith', '{"spotify": "2bA2YuQk2ID3PWNXUwXPAf"}'::jsonb, '{"genres": ["country"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Toby Keith');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jason Aldean', '{"spotify": "4V8Sr092TqfHkfAA5gXjZ4"}'::jsonb, '{"genres": ["country"], "popularity": 78}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jason Aldean');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Luke Bryan', '{"spotify": "0BvkDsjIUla7X0k6CSWh1I"}'::jsonb, '{"genres": ["country"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Luke Bryan');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Kane Brown', '{"spotify": "3oSJ7TBVCWMTQu0oBpovH8"}'::jsonb, '{"genres": ["country"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Kane Brown');

-- More R&B Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Chris Stokes', '{}'::jsonb, '{"genres": ["r&b"], "note": "Producer/manager", "popularity": 25}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Chris Stokes');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Omarion', '{"spotify": "7p8sWzAl0AYxWRE7zFTCqF"}'::jsonb, '{"genres": ["r&b", "hip hop"], "popularity": 62}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Omarion');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'B2K', '{"spotify": "2KftmGt38cD5HrdLJDrA8U"}'::jsonb, '{"genres": ["r&b"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'B2K');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Raz-B', '{}'::jsonb, '{"genres": ["r&b"], "note": "B2K member", "popularity": 25}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Raz-B');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ne-Yo', '{"spotify": "21E3waRsmPlU7jZsS13rcj"}'::jsonb, '{"genres": ["r&b", "pop"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ne-Yo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'August Alsina', '{"spotify": "5ZS223C6JyBfXasXxrRqOk"}'::jsonb, '{"genres": ["r&b", "hip hop"], "popularity": 62}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'August Alsina');

-- =============================================
-- PART 2: INSERT OFFENSES FOR NEW ARTISTS
-- =============================================

-- Seungri
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'egregious'::offense_severity,
       'Burning Sun Scandal Conviction',
       'Convicted in Burning Sun nightclub scandal involving prostitution mediation and illegal gambling. Sentenced to 3 years in military prison.',
       '2019-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Seungri'
ON CONFLICT DO NOTHING;

-- Jung Joon-young
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'egregious'::offense_severity,
       'Hidden Camera Sex Crimes',
       'Convicted of filming and distributing illegal sex videos without consent in group chat with other celebrities. Sentenced to 6 years in prison.',
       '2019-03-21', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jung Joon-young'
ON CONFLICT DO NOTHING;

-- Kris Wu
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'egregious'::offense_severity,
       'Rape Conviction in China',
       'Convicted of rape in China. Sentenced to 13 years in prison plus deportation. Multiple women accused him of using alcohol to assault them.',
       '2022-11-25', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Kris Wu'
ON CONFLICT DO NOTHING;

-- Bassnectar
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Sexual Misconduct with Minors Allegations',
       'Multiple women alleged he engaged in sexual relationships when they were minors. Sued by two women. Stepped away from music after allegations.',
       '2020-07-03', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Bassnectar'
ON CONFLICT DO NOTHING;

-- Dappy
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Assault Convictions',
       'Multiple assault convictions including hitting man with tennis racket and spitting at women in petrol station. Community service sentences.',
       '2012-02-06', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Dappy'
ON CONFLICT DO NOTHING;

-- Giggs
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Firearms Conviction',
       'Convicted of possession of firearm and ammunition. Served 2 years in prison. Has since maintained clean record.',
       '2003-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Giggs'
ON CONFLICT DO NOTHING;

-- Gene Simmons
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'moderate'::offense_severity,
       'Sexual Misconduct Allegations',
       'Accused of groping reporter during interview. Banned from Fox News. Also accused of unwanted sexual advances by multiple women.',
       '2017-12-13', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Gene Simmons'
ON CONFLICT DO NOTHING;

-- Scott Weiland
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Drug Possession Arrests',
       'Multiple arrests for drug possession throughout career including heroin and cocaine. Died from drug overdose in 2015.',
       '1995-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Scott Weiland'
ON CONFLICT DO NOTHING;

-- Farruko
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Currency Smuggling',
       'Arrested at San Juan airport for failing to declare over $50,000 in cash. Pleaded guilty to currency smuggling. Sentenced to probation.',
       '2015-05-08', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Farruko'
ON CONFLICT DO NOTHING;

-- Don Omar
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'moderate'::offense_severity,
       'Domestic Violence Arrest',
       'Arrested on domestic violence charges in Puerto Rico. Wife alleged abuse. Charges were later dropped.',
       '2021-12-19', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Don Omar'
ON CONFLICT DO NOTHING;

-- Jason Aldean
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'racism'::offense_category, 'moderate'::offense_severity,
       'Controversial "Try That in a Small Town" Video',
       'Music video featured imagery from BLM protests and was filmed at site of 1927 lynching. Accused of promoting vigilante justice and racism. CMT pulled video.',
       '2023-07-14', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jason Aldean'
ON CONFLICT DO NOTHING;

-- Chris Stokes / B2K
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'severe'::offense_severity,
       'Child Molestation Allegations',
       'Accused by B2K members Raz-B and others of sexual abuse when they were minors. Multiple accusers came forward over the years. Never charged.',
       '2007-12-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Chris Stokes'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 3: INSERT EVIDENCE FOR NEW OFFENSES
-- =============================================

-- Kris Wu evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.bbc.com/news/world-asia-china-63755197',
       'BBC News', 'news',
       'Kris Wu: Chinese-Canadian pop star sentenced to 13 years for rape',
       '2022-11-25', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Kris Wu' AND ao.title = 'Rape Conviction in China'
ON CONFLICT DO NOTHING;

-- Seungri evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.bbc.com/news/world-asia-58226166',
       'BBC News', 'news',
       'Seungri: K-pop star sentenced to three years for arranging prostitution',
       '2021-08-12', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Seungri' AND ao.title = 'Burning Sun Scandal Conviction'
ON CONFLICT DO NOTHING;

-- Jung Joon-young evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.bbc.com/news/world-asia-50593670',
       'BBC News', 'news',
       'Jung Joon-young jailed for six years for rape and sharing footage',
       '2019-11-29', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Jung Joon-young' AND ao.title = 'Hidden Camera Sex Crimes'
ON CONFLICT DO NOTHING;

-- Bassnectar evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.billboard.com/music/music-news/bassnectar-sexual-misconduct-allegations-steps-back-music-9415417/',
       'Billboard', 'news',
       'Bassnectar Steps Back From Music Following Sexual Misconduct Allegations',
       '2020-07-04', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Bassnectar' AND ao.title = 'Sexual Misconduct with Minors Allegations'
ON CONFLICT DO NOTHING;

-- Jason Aldean evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2023/07/19/arts/music/jason-aldean-song-small-town-cmt.html',
       'New York Times', 'news',
       'CMT Pulls Jason Aldean Video Amid Outcry Over Imagery',
       '2023-07-19', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Jason Aldean' AND ao.title = 'Controversial "Try That in a Small Town" Video'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 4: INSERT PLATFORM IDS
-- =============================================

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4q3ewBCX7sLwd24euuV69X', 'verified', 1.0
FROM artists WHERE canonical_name = 'Bad Bunny'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1vyhD5VmyZ7KMfW5gqLgo5', 'verified', 1.0
FROM artists WHERE canonical_name = 'J Balvin'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4VMYDCV2IEDYJArk749S6m', 'verified', 1.0
FROM artists WHERE canonical_name = 'Daddy Yankee'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0OdUWJ0sBjDrqHygGUXeCF', 'verified', 1.0
FROM artists WHERE canonical_name = 'Arcangel'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '329e4yvIujISKGKz1BZZbO', 'verified', 1.0
FROM artists WHERE canonical_name = 'Farruko'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5vJFZJbGHhH7aXLqQ9dG3Q', 'verified', 1.0
FROM artists WHERE canonical_name = 'Kris Wu'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4E1BVt1FLVvN9kV7aSrSoa', 'verified', 1.0
FROM artists WHERE canonical_name = 'Dappy'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5Y5TRrQiqgUO4S36tzjIRZ', 'verified', 1.0
FROM artists WHERE canonical_name = 'Giggs'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2p1fiYHYiXz9qi0JJyxBzN', 'verified', 1.0
FROM artists WHERE canonical_name = 'Skepta'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2SrSdSvpminqmStGELCSNd', 'verified', 1.0
FROM artists WHERE canonical_name = 'Stormzy'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '07XSN3sPlIlB2L2XNcTwJw', 'verified', 1.0
FROM artists WHERE canonical_name = 'KISS'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '73YYnb9vLiWZ5U1EGZj65v', 'verified', 1.0
FROM artists WHERE canonical_name = 'Bassnectar'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0QHgL1lAIqAw0HtD7YldmP', 'verified', 1.0
FROM artists WHERE canonical_name = 'DJ Khaled'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '64KEffDW9EtZ1y2vBYgq8T', 'verified', 1.0
FROM artists WHERE canonical_name = 'Marshmello'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4V8Sr092TqfHkfAA5gXjZ4', 'verified', 1.0
FROM artists WHERE canonical_name = 'Jason Aldean'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0BvkDsjIUla7X0k6CSWh1I', 'verified', 1.0
FROM artists WHERE canonical_name = 'Luke Bryan'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3oSJ7TBVCWMTQu0oBpovH8', 'verified', 1.0
FROM artists WHERE canonical_name = 'Kane Brown'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '21E3waRsmPlU7jZsS13rcj', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ne-Yo'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5ZS223C6JyBfXasXxrRqOk', 'verified', 1.0
FROM artists WHERE canonical_name = 'August Alsina'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '64tNsm6TnZe2zpcMVMOoHL', 'verified', 1.0
FROM artists WHERE canonical_name = 'Alice in Chains'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2UazAtjfzqBF0Nho2awK4z', 'verified', 1.0
FROM artists WHERE canonical_name = 'Stone Temple Pilots'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2bA2YuQk2ID3PWNXUwXPAf', 'verified', 1.0
FROM artists WHERE canonical_name = 'Toby Keith'
ON CONFLICT (artist_id, platform) DO NOTHING;
