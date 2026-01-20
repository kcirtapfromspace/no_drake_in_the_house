-- Migration: 031_seed_artists_batch4_drugs_fraud.sql
-- Batch 4: Drug Trafficking, Fraud, Financial Crimes, and Animal Abuse Artists (~80 artists)

-- =============================================
-- PART 1: INSERT NEW ARTISTS - Drug Trafficking Category
-- =============================================

-- Major RICO/Drug Trafficking Cases
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Yo Gotti', '{"spotify": "6TdxjMHuYLiGwu6D1KmXhZ"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Yo Gotti');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Moneybagg Yo', '{"spotify": "3VDKXCjOp3A7SjW4J0wvZY"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Moneybagg Yo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Dolph', '{"spotify": "3FOhL6AKdGgGdMPxGDwh1F"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Dolph');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Key Glock', '{"spotify": "3DWqcuprUgcjGrhpHJkXMv"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Key Glock');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Slimelife Shawty', '{"spotify": "0FvTdv7XrkjVXH3I5KVh1N"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Slimelife Shawty');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Yachty', '{"spotify": "6icQOAFXDZKsumw3YXyusw"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Yachty');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'YFN Lucci', '{"spotify": "0v5JIXbvtcKs7W7j50fI0v"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'YFN Lucci');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tay Keith', '{"spotify": "1FVkwAfD87lUO5pPQLvjCl"}'::jsonb, '{"genres": ["hip hop", "trap"], "note": "Producer", "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tay Keith');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Peewee Longway', '{"spotify": "0Kp93W45F3N9cghS5c4bFC"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Peewee Longway');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Duke', '{}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Duke');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'YSL Records', '{}'::jsonb, '{"genres": ["hip hop", "trap"], "note": "Young Stoner Life Records", "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'YSL Records');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Slimelife Shawty', '{"spotify": "0FvTdv7XrkjVXH3I5KVh1N"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Slimelife Shawty');

-- Drug Convictions
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'G-Unit', '{"spotify": "6Xgsr8vR7v7a0WHUhWQqzN"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'G-Unit');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Juelz Santana', '{"spotify": "4qU9cpB4WwQdyqLB3K5Vxg"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 58}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Juelz Santana');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Kim', '{"spotify": "5tth2a3v0sWwV1C7bApBdX"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 68}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Kim');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Foxy Brown', '{"spotify": "7h0Gm6GD3sPqbPbwTb5jO3"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Foxy Brown');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Beanie Sigel', '{"spotify": "3cxv1jCT5GcfMVgFbAQHVA"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Beanie Sigel');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Cassidy', '{"spotify": "7n8vDo6c2a5m2qnYtDFg4R"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Cassidy');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Memphis Bleek', '{"spotify": "7ICrlKBVTgPi2utYYPfTG9"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Memphis Bleek');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Prodigy', '{"spotify": "18cKkptadWR4QL5hvTEYnx"}'::jsonb, '{"genres": ["hip hop", "rap"], "note": "Mobb Deep member", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Prodigy');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Havoc', '{"spotify": "3AOSHvQE8QnNAUBprqGZT7"}'::jsonb, '{"genres": ["hip hop", "rap"], "note": "Mobb Deep member", "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Havoc');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Mobb Deep', '{"spotify": "0HJVTujzMWqb47FBEyb77t"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 62}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Mobb Deep');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Capone-N-Noreaga', '{"spotify": "5TJfKKl4zCYMhknRuSiUiQ"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 48}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Capone-N-Noreaga');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Big Pun', '{"spotify": "05Bv4Jxptzy9X2mJwWkQfy"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Big Pun');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tony Yayo', '{"spotify": "2kKhv8TyK0PmKjNxvX24E5"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tony Yayo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lloyd Banks', '{"spotify": "5PaEXZBB5LJBUGSprBMLM8"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lloyd Banks');

-- International Drug Cases
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Pitbull', '{"spotify": "0TnOYISbd1XYRBk9myaseg"}'::jsonb, '{"genres": ["hip hop", "dance"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Pitbull');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Snoop Dogg', '{"spotify": "7JVmFSIFLGHX0ZAb2xTPhO"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 90}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Snoop Dogg');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Willie Nelson', '{"spotify": "5W5bDNCqJ1jbCgTxDD0Cb3"}'::jsonb, '{"genres": ["country", "outlaw country"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Willie Nelson');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tommy Chong', '{"spotify": "4UB8CpPa05xtBvJF0t24Ml"}'::jsonb, '{"genres": ["comedy", "rock"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tommy Chong');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Fiona Apple', '{"spotify": "3g2kUQ6tHLLbmkV7T4GPtL"}'::jsonb, '{"genres": ["alternative", "indie"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Fiona Apple');

-- =============================================
-- FRAUD/FINANCIAL CRIMES
-- =============================================

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Vanilla Ice', '{"spotify": "7HxQxOlhUHlVzLQz8ZXCPD"}'::jsonb, '{"genres": ["hip hop", "pop rap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Vanilla Ice');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'MC Hammer', '{"spotify": "0Y6m8lJd1tHgHPgNIk2RnB"}'::jsonb, '{"genres": ["hip hop", "pop rap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'MC Hammer');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Wesley Snipes', '{"spotify": "5e7r8jXMWRNmKTNmL4jTJR"}'::jsonb, '{"genres": ["pop"], "note": "Actor/singer", "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Wesley Snipes');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Billy Joel', '{"spotify": "6zFYqv1mProcEC7YdW1Tj5"}'::jsonb, '{"genres": ["rock", "pop"], "popularity": 78}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Billy Joel');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'TLC', '{"spotify": "0TImkz4nPqjegtVSMZnMRq"}'::jsonb, '{"genres": ["r&b", "hip hop"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'TLC');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Left Eye', '{}'::jsonb, '{"genres": ["r&b", "hip hop"], "note": "TLC member Lisa Lopes", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Left Eye');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Mase', '{"spotify": "5xBfyV0tBQz4X8wn9j8RNQ"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Mase');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Nas X', '{"spotify": "7jVv8c5Fj3E9VhNjxT4snq"}'::jsonb, '{"genres": ["hip hop", "country", "pop"], "popularity": 88}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Nas X');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ozuna', '{"spotify": "1i8SpTcr7yvPOmcqrbnVXY"}'::jsonb, '{"genres": ["reggaeton", "latin trap"], "popularity": 85}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ozuna');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Anuel AA', '{"spotify": "2R21vXR83lH98kGeO99Y66"}'::jsonb, '{"genres": ["reggaeton", "latin trap"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Anuel AA');

-- =============================================
-- ANIMAL ABUSE
-- =============================================

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Michael Vick', '{}'::jsonb, '{"genres": [], "note": "NFL player with music involvement", "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Michael Vick');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Steven Seagal', '{"spotify": "4Sh8e5T5pM0mPNGpM1PiPY"}'::jsonb, '{"genres": ["blues", "country"], "popularity": 30}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Steven Seagal');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'GG Allin', '{}'::jsonb, '{"genres": ["punk rock"], "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'GG Allin');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Insane Clown Posse', '{"spotify": "2PVNLCMYwPzb7w4xv5RqYp"}'::jsonb, '{"genres": ["horrorcore", "hip hop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Insane Clown Posse');

-- =============================================
-- PART 2: INSERT OFFENSES FOR NEW ARTISTS
-- =============================================

-- YFN Lucci
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'RICO Murder Charges',
       'Indicted on RICO charges including felony murder, aggravated assault, and gang activity. Surrendered to police in 2021. Awaiting trial.',
       '2021-01-13', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'YFN Lucci'
ON CONFLICT DO NOTHING;

-- Juelz Santana
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Gun and Drug Possession at Airport',
       'Fled Newark Airport after gun found in bag. Charged with weapons and drug possession. Pleaded guilty, sentenced to 27 months in federal prison.',
       '2018-03-09', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Juelz Santana'
ON CONFLICT DO NOTHING;

-- Lil Kim
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'fraud'::offense_category, 'moderate'::offense_severity,
       'Perjury Conviction',
       'Convicted of three counts of conspiracy and one count of perjury for lying to grand jury about shooting outside radio station. Served 10 months in prison.',
       '2005-03-17', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Lil Kim'
ON CONFLICT DO NOTHING;

-- Beanie Sigel
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Tax Evasion Conviction',
       'Convicted of federal income tax evasion for not filing returns on over $1.4 million income. Sentenced to 2 years in federal prison.',
       '2012-08-22', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Beanie Sigel'
ON CONFLICT DO NOTHING;

-- Prodigy
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Gun Possession Conviction',
       'Arrested for criminal possession of a weapon after police found loaded .22 caliber gun in car. Served 3.5 years in prison.',
       '2006-09-11', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Prodigy'
ON CONFLICT DO NOTHING;

-- Vanilla Ice
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'fraud'::offense_category, 'minor'::offense_severity,
       'Burglary and Grand Theft Charges',
       'Arrested for stealing furniture, pool equipment, and bicycles from foreclosed home next to property he was renovating. Took plea deal.',
       '2015-02-18', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Vanilla Ice'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'moderate'::offense_severity,
       'Domestic Violence Arrest',
       'Arrested for domestic battery after wife called police alleging he grabbed and pushed her. Charges eventually dropped.',
       '2008-11-11', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Vanilla Ice'
ON CONFLICT DO NOTHING;

-- MC Hammer
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'fraud'::offense_category, 'moderate'::offense_severity,
       'Bankruptcy Fraud Investigation',
       'Filed for bankruptcy owing $13 million after spending extravagantly. Investigated for fraud but not charged. Lost nearly entire fortune.',
       '1996-04-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'MC Hammer'
ON CONFLICT DO NOTHING;

-- Wesley Snipes
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'fraud'::offense_category, 'severe'::offense_severity,
       'Tax Evasion Conviction',
       'Convicted of three counts of willful failure to file federal income tax returns. Owed approximately $7 million. Served 3 years in federal prison.',
       '2008-02-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Wesley Snipes'
ON CONFLICT DO NOTHING;

-- TLC/Left Eye
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Arson - Burning Boyfriend''s House',
       'Lisa "Left Eye" Lopes burned down boyfriend Andre Rison''s mansion by setting fire to his sneakers in bathtub. Pleaded guilty to arson.',
       '1994-06-09', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'TLC'
ON CONFLICT DO NOTHING;

-- Anuel AA
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Federal Gun Charges',
       'Arrested for illegal gun possession in Puerto Rico. Pleaded guilty to federal firearms charges. Served 30 months in federal prison.',
       '2016-04-03', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Anuel AA'
ON CONFLICT DO NOTHING;

-- Willie Nelson
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'minor'::offense_severity,
       'Multiple Marijuana Arrests',
       'Arrested numerous times for marijuana possession throughout career. Most famously arrested in Texas in 2010 with 6 ounces on tour bus.',
       '2010-11-26', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Willie Nelson'
ON CONFLICT DO NOTHING;

-- Tommy Chong
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Selling Drug Paraphernalia',
       'Convicted of conspiracy to distribute drug paraphernalia (bongs) through Nice Dreams company. Served 9 months in federal prison.',
       '2003-09-11', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tommy Chong'
ON CONFLICT DO NOTHING;

-- Michael Vick
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'animal_abuse'::offense_category, 'egregious'::offense_severity,
       'Dog Fighting Operation',
       'Pleaded guilty to federal charges related to dog fighting operation "Bad Newz Kennels." Dogs tortured and killed. Served 21 months in federal prison.',
       '2007-08-27', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Michael Vick'
ON CONFLICT DO NOTHING;

-- GG Allin
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Sexual Assault Conviction',
       'Pleaded guilty to assault with intent to do great bodily harm less than murder after woman accused him of assault. Served 2 years in prison.',
       '1991-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'GG Allin'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Multiple Assault Arrests',
       'Notorious for violent performances including throwing feces, assaulting audience members, and self-mutilation. Multiple arrests for assault and indecent exposure.',
       '1989-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'GG Allin'
ON CONFLICT DO NOTHING;

-- Yo Gotti
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Drug and Gun Charges History',
       'Multiple arrests for drug possession and weapons charges. Associates connected to cocaine trafficking. No major convictions.',
       '2006-01-01', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Yo Gotti'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 3: INSERT EVIDENCE FOR NEW OFFENSES
-- =============================================

-- YFN Lucci evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.ajc.com/news/crime/rapper-yfn-lucci-surrenders-on-murder-charge/JQBTFYJ2LRBKXP2XYUCFTHSUWI/',
       'Atlanta Journal-Constitution', 'news',
       'Rapper YFN Lucci Surrenders on Murder Charge',
       '2021-01-13', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'YFN Lucci' AND ao.title = 'RICO Murder Charges'
ON CONFLICT DO NOTHING;

-- Juelz Santana evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2018/12/19/nyregion/juelz-santana-sentencing-gun-airport.html',
       'New York Times', 'news',
       'Juelz Santana Sentenced to 27 Months for Gun at Airport',
       '2018-12-19', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Juelz Santana' AND ao.title = 'Gun and Drug Possession at Airport'
ON CONFLICT DO NOTHING;

-- Lil Kim evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2005/03/18/arts/music/lil-kim-convicted-of-conspiracy-and-perjury.html',
       'New York Times', 'news',
       'Lil'' Kim Convicted of Conspiracy and Perjury',
       '2005-03-18', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Lil Kim' AND ao.title = 'Perjury Conviction'
ON CONFLICT DO NOTHING;

-- Wesley Snipes evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2010/12/10/business/10snipes.html',
       'New York Times', 'news',
       'Wesley Snipes Begins 3-Year Prison Sentence',
       '2010-12-10', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Wesley Snipes' AND ao.title = 'Tax Evasion Conviction'
ON CONFLICT DO NOTHING;

-- Michael Vick evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2007/08/28/sports/football/28vick.html',
       'New York Times', 'news',
       'Vick Pleads Guilty in Dog-Fighting Case',
       '2007-08-28', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Michael Vick' AND ao.title = 'Dog Fighting Operation'
ON CONFLICT DO NOTHING;

-- Tommy Chong evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/culture/culture-news/tommy-chong-begins-jail-sentence-93099/',
       'Rolling Stone', 'news',
       'Tommy Chong Begins Jail Sentence',
       '2003-10-09', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Tommy Chong' AND ao.title = 'Selling Drug Paraphernalia'
ON CONFLICT DO NOTHING;

-- Anuel AA evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.billboard.com/music/latin/anuel-aa-released-prison-8452193/',
       'Billboard', 'news',
       'Anuel AA Released from Prison',
       '2018-07-17', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Anuel AA' AND ao.title = 'Federal Gun Charges'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 4: INSERT PLATFORM IDS
-- =============================================

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6TdxjMHuYLiGwu6D1KmXhZ', 'verified', 1.0
FROM artists WHERE canonical_name = 'Yo Gotti'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0v5JIXbvtcKs7W7j50fI0v', 'verified', 1.0
FROM artists WHERE canonical_name = 'YFN Lucci'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4qU9cpB4WwQdyqLB3K5Vxg', 'verified', 1.0
FROM artists WHERE canonical_name = 'Juelz Santana'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5tth2a3v0sWwV1C7bApBdX', 'verified', 1.0
FROM artists WHERE canonical_name = 'Lil Kim'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3cxv1jCT5GcfMVgFbAQHVA', 'verified', 1.0
FROM artists WHERE canonical_name = 'Beanie Sigel'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0HJVTujzMWqb47FBEyb77t', 'verified', 1.0
FROM artists WHERE canonical_name = 'Mobb Deep'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7HxQxOlhUHlVzLQz8ZXCPD', 'verified', 1.0
FROM artists WHERE canonical_name = 'Vanilla Ice'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0Y6m8lJd1tHgHPgNIk2RnB', 'verified', 1.0
FROM artists WHERE canonical_name = 'MC Hammer'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0TImkz4nPqjegtVSMZnMRq', 'verified', 1.0
FROM artists WHERE canonical_name = 'TLC'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2R21vXR83lH98kGeO99Y66', 'verified', 1.0
FROM artists WHERE canonical_name = 'Anuel AA'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5W5bDNCqJ1jbCgTxDD0Cb3', 'verified', 1.0
FROM artists WHERE canonical_name = 'Willie Nelson'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6icQOAFXDZKsumw3YXyusw', 'verified', 1.0
FROM artists WHERE canonical_name = 'Lil Yachty'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2PVNLCMYwPzb7w4xv5RqYp', 'verified', 1.0
FROM artists WHERE canonical_name = 'Insane Clown Posse'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6zFYqv1mProcEC7YdW1Tj5', 'verified', 1.0
FROM artists WHERE canonical_name = 'Billy Joel'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3g2kUQ6tHLLbmkV7T4GPtL', 'verified', 1.0
FROM artists WHERE canonical_name = 'Fiona Apple'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0TnOYISbd1XYRBk9myaseg', 'verified', 1.0
FROM artists WHERE canonical_name = 'Pitbull'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3DWqcuprUgcjGrhpHJkXMv', 'verified', 1.0
FROM artists WHERE canonical_name = 'Key Glock'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1i8SpTcr7yvPOmcqrbnVXY', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ozuna'
ON CONFLICT (artist_id, platform) DO NOTHING;
