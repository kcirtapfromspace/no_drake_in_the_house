-- Migration: 029_seed_artists_batch2_misconduct.sql
-- Batch 2: Sexual Misconduct, Domestic Violence, and Child Abuse Artists (~80 artists)

-- =============================================
-- PART 1: INSERT NEW ARTISTS - Sexual Misconduct Category
-- =============================================

-- Historical Sexual Misconduct Cases
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jerry Lee Lewis', '{"spotify": "2zyz0VJqrDXeFDIyrfVXSo"}'::jsonb, '{"genres": ["rock and roll", "country", "rockabilly"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jerry Lee Lewis');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Chuck Berry', '{"spotify": "293zczrfYafIItmnmM3coR"}'::jsonb, '{"genres": ["rock and roll", "blues"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Chuck Berry');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bill Wyman', '{"spotify": "4PFYc5hy3VLGxZKnQRZ7zP"}'::jsonb, '{"genres": ["rock"], "note": "Rolling Stones bassist", "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bill Wyman');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Steven Tyler', '{"spotify": "77AiFEVeAVj2ORpC85QVJs"}'::jsonb, '{"genres": ["rock", "hard rock"], "note": "Aerosmith vocalist", "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Steven Tyler');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jimmy Page', '{"spotify": "4bPOOw8BYjXdp3CdffETSI"}'::jsonb, '{"genres": ["rock", "blues rock"], "note": "Led Zeppelin guitarist", "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jimmy Page');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'David Bowie', '{"spotify": "0oSGxfWSnnOXhD2fKuz2Gy"}'::jsonb, '{"genres": ["rock", "glam rock", "art rock"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'David Bowie');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Iggy Pop', '{"spotify": "36E7oYfz3LLRto6yCpf85B"}'::jsonb, '{"genres": ["rock", "punk rock"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Iggy Pop');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Pete Townshend', '{"spotify": "4rWGNHxOdmm5mSYLfZcnYo"}'::jsonb, '{"genres": ["rock"], "note": "The Who guitarist", "popularity": 58}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Pete Townshend');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Michael Jackson', '{"spotify": "3fMbdgg4jU18AjLCKBhRSm"}'::jsonb, '{"genres": ["pop", "r&b", "soul"], "popularity": 90}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Michael Jackson');

-- Contemporary Sexual Misconduct Cases
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ryan Adams', '{"spotify": "2qc41rNTtdLK0tV3mS8UHb"}'::jsonb, '{"genres": ["rock", "alt-country"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ryan Adams');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jesse Lacey', '{"spotify": "2uoj6gFHNG6sGPhBbPz9x3"}'::jsonb, '{"genres": ["rock", "emo"], "note": "Brand New vocalist", "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jesse Lacey');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Aaron Lewis', '{"spotify": "7Jjl9Vl6GSPFaAHCGfkbcS"}'::jsonb, '{"genres": ["rock", "country"], "note": "Staind vocalist", "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Aaron Lewis');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jacob Hoggard', '{}'::jsonb, '{"genres": ["pop rock"], "note": "Hedley vocalist", "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jacob Hoggard');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Hedley', '{"spotify": "6sFIWsNpZYqfjUpaCgueju"}'::jsonb, '{"genres": ["pop rock", "rock"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Hedley');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Trey Songz', '{"spotify": "2iojnBLj0qIMiKPvVhLnsH"}'::jsonb, '{"genres": ["r&b", "hip hop"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Trey Songz');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jeremih', '{"spotify": "7sfl4Xt5KmfyDs2T3SVSMK"}'::jsonb, '{"genres": ["r&b", "hip hop"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jeremih');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Brand Nubian', '{"spotify": "1x7RQWPJ1TKH1VOMdWmHrR"}'::jsonb, '{"genres": ["hip hop"], "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Brand Nubian');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Necro', '{"spotify": "2kCcBybjl3SAtIcwdWpUe3"}'::jsonb, '{"genres": ["hip hop", "horrorcore"], "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Necro');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'CeeLo Green', '{"spotify": "5nLYd9ST4Cnwy6NHaCxbj8"}'::jsonb, '{"genres": ["r&b", "soul"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'CeeLo Green');

-- Domestic Violence Cases
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bobby Brown', '{"spotify": "7FsGSmrqBpgMKm7sLN6vMk"}'::jsonb, '{"genres": ["r&b", "new jack swing"], "popularity": 62}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bobby Brown');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jason Derulo', '{"spotify": "07YZf4WDAMNwqr4jfgOZ8y"}'::jsonb, '{"genres": ["pop", "r&b"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jason Derulo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Usher', '{"spotify": "23zg3TcAtWQy7J6upgbUnj"}'::jsonb, '{"genres": ["r&b", "pop"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Usher');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tyga', '{"spotify": "5LHRHt1k9lMyONurDHEdrp"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 78}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tyga');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bow Wow', '{"spotify": "5X7jkPoZ3jZMUYdV8zq7Bj"}'::jsonb, '{"genres": ["hip hop", "r&b"], "popularity": 62}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bow Wow');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Big Sean', '{"spotify": "0c173mlxpT3dSFRgMO8XPh"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Big Sean');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Future', '{"spotify": "1RyvyyTE3xzB2ZywiAwp0i"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 90}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Future');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Rich the Kid', '{"spotify": "1pPmIToKXyGdsCF6LmqLmI"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Rich the Kid');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Soulja Boy', '{"spotify": "2kmF4yEO7kqmDUJVUhxk8f"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Soulja Boy');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Scott Stapp', '{"spotify": "1h8Fq1u2YvSY0mDwKHNaS9"}'::jsonb, '{"genres": ["rock", "post-grunge"], "note": "Creed vocalist", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Scott Stapp');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Brendon Urie', '{"spotify": "4LG4Bs1Gadht7TCrMytQUO"}'::jsonb, '{"genres": ["pop rock", "alternative"], "note": "Panic! at the Disco", "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Brendon Urie');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Yung Berg', '{"spotify": "0gLvDFx0lPjUZo5FjRVOJj"}'::jsonb, '{"genres": ["hip hop", "r&b"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Yung Berg');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Pimp C', '{"spotify": "1SJOL9HJ08YOn92lFcYf8a"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "note": "UGK member", "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Pimp C');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Big Boi', '{"spotify": "5qzkmMs9rPQVVPuzNWXANk"}'::jsonb, '{"genres": ["hip hop", "rap"], "note": "OutKast member", "popularity": 68}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Big Boi');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Trouble', '{"spotify": "3mNwNBHhb4q2q9jNPvJyjt"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Trouble');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Slim 400', '{"spotify": "5xDtLR35bVcmYhVYJ0RoAl"}'::jsonb, '{"genres": ["hip hop", "west coast rap"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Slim 400');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tory Lanez', '{"spotify": "2jku7tDXc6XoB6MO2hFuqg"}'::jsonb, '{"genres": ["hip hop", "r&b"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tory Lanez');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jonathan Davis', '{"spotify": "3VUTk6H4C9dH7sR2LXv3hM"}'::jsonb, '{"genres": ["nu metal", "metal"], "note": "Korn vocalist", "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jonathan Davis');

-- Child Abuse Related Cases
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Xavier Naidoo', '{"spotify": "1gfWPDuDK3Db1IKaXwrQ6L"}'::jsonb, '{"genres": ["r&b", "soul", "pop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Xavier Naidoo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Mark Salling', '{}'::jsonb, '{"genres": ["pop", "rock"], "note": "Glee actor/singer", "popularity": 30}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Mark Salling');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jared Leto', '{"spotify": "7LnaAXbDVIL7eL7s42QYI3"}'::jsonb, '{"genres": ["rock", "alternative"], "note": "Thirty Seconds to Mars", "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jared Leto');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Danny Masterson', '{}'::jsonb, '{"genres": ["rock"], "note": "Actor/DJ", "popularity": 25}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Danny Masterson');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Kip Winger', '{"spotify": "3r4Ng1Sz05HyPACrPYJdiv"}'::jsonb, '{"genres": ["glam metal", "rock"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Kip Winger');

-- More Recent Sexual Misconduct Cases
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Marilyn Manson', '{"spotify": "2VYQTNDsvvKN9wmU5W7xpj"}'::jsonb, '{"genres": ["industrial rock", "metal"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Marilyn Manson');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Limp Bizkit', '{"spotify": "165ZgPlLkK7bf5bDoFc6Sb"}'::jsonb, '{"genres": ["nu metal", "rap rock"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Limp Bizkit');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Fred Durst', '{"spotify": "6YxO6djt3aDsMglPLQr3wa"}'::jsonb, '{"genres": ["nu metal", "rap rock"], "note": "Limp Bizkit vocalist", "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Fred Durst');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Orgy', '{"spotify": "6BV9wY8uPEQMwHGqFNPJDg"}'::jsonb, '{"genres": ["industrial rock", "nu metal"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Orgy');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jay Gordon', '{}'::jsonb, '{"genres": ["industrial rock", "nu metal"], "note": "Orgy vocalist", "popularity": 30}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jay Gordon');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Die Antwoord', '{"spotify": "2wGX7MUcq2JNVbTtHFHsGj"}'::jsonb, '{"genres": ["hip hop", "electronic"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Die Antwoord');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ninja', '{}'::jsonb, '{"genres": ["hip hop", "electronic"], "note": "Die Antwoord member", "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ninja');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'PWR BTTM', '{}'::jsonb, '{"genres": ["indie rock", "queercore"], "popularity": 25}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'PWR BTTM');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ben Hopkins', '{}'::jsonb, '{"genres": ["indie rock"], "note": "PWR BTTM member", "popularity": 20}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ben Hopkins');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Front Porch Step', '{}'::jsonb, '{"genres": ["acoustic", "emo"], "popularity": 25}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Front Porch Step');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jake McElfresh', '{}'::jsonb, '{"genres": ["acoustic", "emo"], "note": "Front Porch Step", "popularity": 20}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jake McElfresh');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Dahvie Vanity', '{}'::jsonb, '{"genres": ["crunkcore", "electronic"], "note": "Blood on the Dance Floor", "popularity": 25}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Dahvie Vanity');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Blood on the Dance Floor', '{"spotify": "73GCZlf6pPNlMRnLuPkqIx"}'::jsonb, '{"genres": ["crunkcore", "electronic"], "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Blood on the Dance Floor');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Andy Biersack', '{"spotify": "6FQyUYJjCuBZl3GezEgw1W"}'::jsonb, '{"genres": ["rock", "glam metal"], "note": "Black Veil Brides vocalist", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Andy Biersack');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ronnie Radke', '{"spotify": "5mqt3kAWM6e6EqSjzC3KJv"}'::jsonb, '{"genres": ["rock", "metalcore"], "note": "Falling in Reverse vocalist", "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ronnie Radke');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Austin Jones', '{}'::jsonb, '{"genres": ["pop punk", "acoustic"], "popularity": 20}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Austin Jones');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Kyle Pavone', '{}'::jsonb, '{"genres": ["metalcore"], "note": "We Came as Romans vocalist", "popularity": 30}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Kyle Pavone');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Josh Duggar', '{}'::jsonb, '{"genres": ["christian"], "note": "19 Kids reality TV", "popularity": 20}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Josh Duggar');

-- =============================================
-- PART 2: INSERT OFFENSES FOR NEW ARTISTS
-- =============================================

-- Jerry Lee Lewis
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Marriage to 13-Year-Old Cousin',
       'Married his 13-year-old first cousin once removed, Myra Gale Brown, when he was 22. The marriage caused major scandal and temporarily destroyed his career.',
       '1957-12-12', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jerry Lee Lewis'
ON CONFLICT DO NOTHING;

-- Chuck Berry
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'severe'::offense_severity,
       'Mann Act Conviction',
       'Convicted under the Mann Act for transporting 14-year-old girl across state lines for immoral purposes. Served 20 months in federal prison.',
       '1959-12-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Chuck Berry'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'moderate'::offense_severity,
       'Bathroom Voyeurism Lawsuit',
       'Sued by multiple women for installing secret cameras in restrooms at his restaurant. Settled numerous lawsuits for undisclosed amounts.',
       '1990-01-01', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Chuck Berry'
ON CONFLICT DO NOTHING;

-- Bill Wyman
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Relationship with 13-Year-Old',
       'Began relationship with Mandy Smith when she was 13 and he was 47. They later married when she was 18. Admitted in autobiography.',
       '1984-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Bill Wyman'
ON CONFLICT DO NOTHING;

-- Steven Tyler
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Underage Guardian Relationship',
       'Obtained guardianship of 16-year-old girlfriend Julia Holcomb when he was 25-27. She later alleged abuse and coerced abortion. Sued in 2022.',
       '1975-01-01', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Steven Tyler'
ON CONFLICT DO NOTHING;

-- Jimmy Page
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Relationship with 14-Year-Old',
       'Had three-year relationship with 14-year-old Lori Mattix. She was kept hidden at his home. Documented in groupie histories.',
       '1972-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jimmy Page'
ON CONFLICT DO NOTHING;

-- Michael Jackson
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'severe'::offense_severity,
       'Child Molestation Allegations and Trial',
       'Accused of child molestation by multiple boys. Settled 1993 case for reported $23 million. Acquitted on all counts in 2005 trial. Post-mortem accusations in Leaving Neverland documentary.',
       '1993-08-17', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Michael Jackson'
ON CONFLICT DO NOTHING;

-- Ryan Adams
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'moderate'::offense_severity,
       'Sexual Misconduct Allegations',
       'Multiple women including Phoebe Bridgers and Mandy Moore alleged emotional abuse and sexual misconduct. FBI investigated text exchanges with minor. Career significantly impacted.',
       '2019-02-13', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Ryan Adams'
ON CONFLICT DO NOTHING;

-- Jesse Lacey
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Sexual Misconduct with Minors',
       'Multiple women alleged he solicited nude photos when they were minors and engaged in sexual misconduct. Admitted to treatment for sex addiction.',
       '2017-11-10', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jesse Lacey'
ON CONFLICT DO NOTHING;

-- Jacob Hoggard
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'egregious'::offense_severity,
       'Sexual Assault Conviction',
       'Convicted of sexual assault causing bodily harm. Found guilty of attacking a fan in Toronto hotel room in 2016. Sentenced to 5 years in prison.',
       '2016-06-25', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jacob Hoggard'
ON CONFLICT DO NOTHING;

-- Trey Songz
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Multiple Sexual Assault Allegations',
       'Accused of sexual assault by multiple women in lawsuits. Investigated for rape allegation in Las Vegas. Pattern of allegations spanning years.',
       '2021-01-26', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Trey Songz'
ON CONFLICT DO NOTHING;

-- Tory Lanez
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Shooting of Megan Thee Stallion',
       'Convicted of shooting Megan Thee Stallion in the feet during argument in Hollywood Hills. Found guilty of assault with semiautomatic firearm. Sentenced to 10 years.',
       '2020-07-12', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tory Lanez'
ON CONFLICT DO NOTHING;

-- Danny Masterson
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'egregious'::offense_severity,
       'Rape Convictions',
       'Convicted of two counts of forcible rape involving Scientology members. Sentenced to 30 years to life in California state prison.',
       '2003-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Danny Masterson'
ON CONFLICT DO NOTHING;

-- Dahvie Vanity
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'egregious'::offense_severity,
       'Multiple Underage Sexual Abuse Allegations',
       'Over 20 women accused him of sexual assault, many when they were minors at concerts. Pattern of predatory behavior documented by Huffington Post investigation.',
       '2018-11-13', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Dahvie Vanity'
ON CONFLICT DO NOTHING;

-- Austin Jones
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'egregious'::offense_severity,
       'Child Pornography Conviction',
       'Convicted of producing child pornography by convincing underage fans to send explicit videos. Sentenced to 10 years in federal prison.',
       '2019-05-03', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Austin Jones'
ON CONFLICT DO NOTHING;

-- Mark Salling
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'egregious'::offense_severity,
       'Child Pornography Possession',
       'Pleaded guilty to possessing child pornography, including images of children as young as 3. Was facing 4-7 years. Committed suicide before sentencing.',
       '2017-12-18', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Mark Salling'
ON CONFLICT DO NOTHING;

-- Bobby Brown
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'moderate'::offense_severity,
       'Domestic Violence Arrest',
       'Arrested for domestic violence against wife Whitney Houston. Houston documented abuse in Being Bobby Brown reality show and book.',
       '2003-12-01', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Bobby Brown'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Multiple Drug and DUI Arrests',
       'Numerous arrests for DUI, drug possession, and probation violations throughout career. Lost license multiple times.',
       '1996-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Bobby Brown'
ON CONFLICT DO NOTHING;

-- Soulja Boy
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'moderate'::offense_severity,
       'Domestic Violence Allegation - Kayla Myers',
       'Ex-girlfriend Kayla Myers filed lawsuit alleging physical abuse, kidnapping, and sexual assault. Restraining order issued.',
       '2021-01-22', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Soulja Boy'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Weapons Charges',
       'Arrested for possessing loaded firearm in vehicle. Violated probation from previous case. Served 240 days in jail.',
       '2019-04-11', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Soulja Boy'
ON CONFLICT DO NOTHING;

-- Ronnie Radke
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Battery and Manslaughter',
       'Involved in fight where someone was shot and killed. Convicted of battery with substantial bodily harm. Served 2.5 years in prison.',
       '2006-05-11', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Ronnie Radke'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'moderate'::offense_severity,
       'Domestic Violence Arrest',
       'Arrested for domestic violence. Ex-girlfriend alleged abuse. Multiple incidents of aggressive behavior at concerts including throwing objects at fans.',
       '2012-05-07', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Ronnie Radke'
ON CONFLICT DO NOTHING;

-- Die Antwoord / Ninja
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Child Abuse and Sexual Assault Allegations',
       'Adopted son Tokkie and multiple others alleged physical and sexual abuse. Australian concert canceled. Video emerged of Ninja assaulting Andy Butler.',
       '2019-01-01', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Die Antwoord'
ON CONFLICT DO NOTHING;

-- Josh Duggar
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'egregious'::offense_severity,
       'Child Pornography Conviction',
       'Convicted of receiving child pornography depicting children as young as 18 months. Sentenced to over 12 years in federal prison. Previously admitted to molesting sisters.',
       '2021-04-29', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Josh Duggar'
ON CONFLICT DO NOTHING;

-- Jared Leto
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'moderate'::offense_severity,
       'Multiple Sexual Misconduct Allegations',
       'Multiple women including James Gunn have alleged predatory behavior toward young fans. Pattern of allegations involving underage girls at concerts.',
       '2018-04-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jared Leto'
ON CONFLICT DO NOTHING;

-- Tyga
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'moderate'::offense_severity,
       'Relationship with Underage Kylie Jenner',
       'Began publicly dating Kylie Jenner when she was 17 and he was 24. Relationship sparked controversy and criticism.',
       '2014-10-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tyga'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 3: INSERT EVIDENCE FOR NEW OFFENSES
-- =============================================

-- Jerry Lee Lewis evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/jerry-lee-lewis-marriage-scandal-1234610942/',
       'Rolling Stone', 'news',
       'Jerry Lee Lewis'' 1958 Marriage Scandal',
       '2022-10-28', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Jerry Lee Lewis' AND ao.title = 'Marriage to 13-Year-Old Cousin'
ON CONFLICT DO NOTHING;

-- Michael Jackson evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2005/06/14/arts/music/michael-jackson-acquitted-on-all-counts-in-child.html',
       'New York Times', 'news',
       'Michael Jackson Acquitted on All Counts',
       '2005-06-14', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Michael Jackson' AND ao.title = 'Child Molestation Allegations and Trial'
ON CONFLICT DO NOTHING;

-- Tory Lanez evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2023/08/08/arts/music/tory-lanez-sentenced-megan-thee-stallion.html',
       'New York Times', 'news',
       'Tory Lanez Sentenced to 10 Years for Shooting Megan Thee Stallion',
       '2023-08-08', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Tory Lanez' AND ao.title = 'Shooting of Megan Thee Stallion'
ON CONFLICT DO NOTHING;

-- Danny Masterson evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2023/09/07/arts/television/danny-masterson-sentenced.html',
       'New York Times', 'news',
       'Danny Masterson Sentenced to 30 Years to Life in Prison for Rapes',
       '2023-09-07', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Danny Masterson' AND ao.title = 'Rape Convictions'
ON CONFLICT DO NOTHING;

-- Jacob Hoggard evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.cbc.ca/news/canada/toronto/jacob-hoggard-hedley-sentencing-1.6595765',
       'CBC News', 'news',
       'Jacob Hoggard sentenced to 5 years in prison for sexual assault',
       '2022-10-20', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Jacob Hoggard' AND ao.title = 'Sexual Assault Conviction'
ON CONFLICT DO NOTHING;

-- Austin Jones evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.justice.gov/usao-ndil/pr/youtube-star-austin-jones-sentenced-10-years-federal-prison-persuading-underage-girls',
       'US Department of Justice', 'court',
       'YouTube Star Austin Jones Sentenced to 10 Years',
       '2019-05-03', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Austin Jones' AND ao.title = 'Child Pornography Conviction'
ON CONFLICT DO NOTHING;

-- Ryan Adams evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2019/02/13/arts/music/ryan-adams-women-sex.html',
       'New York Times', 'news',
       'Ryan Adams Dangled Success. Women Say They Paid a Price.',
       '2019-02-13', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Ryan Adams' AND ao.title = 'Sexual Misconduct Allegations'
ON CONFLICT DO NOTHING;

-- Dahvie Vanity evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.huffpost.com/entry/dahvie-vanity-botdf_n_5bea8b27e4b0783e0a1a22f8',
       'HuffPost', 'news',
       'Women Say Dahvie Vanity Assaulted Them for Years',
       '2018-11-13', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Dahvie Vanity' AND ao.title = 'Multiple Underage Sexual Abuse Allegations'
ON CONFLICT DO NOTHING;

-- Josh Duggar evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2022/05/25/us/josh-duggar-child-pornography-sentence.html',
       'New York Times', 'news',
       'Josh Duggar Is Sentenced to Over 12 Years for Child Sex Abuse Images',
       '2022-05-25', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Josh Duggar' AND ao.title = 'Child Pornography Conviction'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 4: INSERT PLATFORM IDS
-- =============================================

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2zyz0VJqrDXeFDIyrfVXSo', 'verified', 1.0
FROM artists WHERE canonical_name = 'Jerry Lee Lewis'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '293zczrfYafIItmnmM3coR', 'verified', 1.0
FROM artists WHERE canonical_name = 'Chuck Berry'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '77AiFEVeAVj2ORpC85QVJs', 'verified', 1.0
FROM artists WHERE canonical_name = 'Steven Tyler'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0oSGxfWSnnOXhD2fKuz2Gy', 'verified', 1.0
FROM artists WHERE canonical_name = 'David Bowie'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3fMbdgg4jU18AjLCKBhRSm', 'verified', 1.0
FROM artists WHERE canonical_name = 'Michael Jackson'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2qc41rNTtdLK0tV3mS8UHb', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ryan Adams'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2iojnBLj0qIMiKPvVhLnsH', 'verified', 1.0
FROM artists WHERE canonical_name = 'Trey Songz'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2jku7tDXc6XoB6MO2hFuqg', 'verified', 1.0
FROM artists WHERE canonical_name = 'Tory Lanez'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7FsGSmrqBpgMKm7sLN6vMk', 'verified', 1.0
FROM artists WHERE canonical_name = 'Bobby Brown'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5LHRHt1k9lMyONurDHEdrp', 'verified', 1.0
FROM artists WHERE canonical_name = 'Tyga'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1RyvyyTE3xzB2ZywiAwp0i', 'verified', 1.0
FROM artists WHERE canonical_name = 'Future'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2kmF4yEO7kqmDUJVUhxk8f', 'verified', 1.0
FROM artists WHERE canonical_name = 'Soulja Boy'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5mqt3kAWM6e6EqSjzC3KJv', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ronnie Radke'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2wGX7MUcq2JNVbTtHFHsGj', 'verified', 1.0
FROM artists WHERE canonical_name = 'Die Antwoord'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7LnaAXbDVIL7eL7s42QYI3', 'verified', 1.0
FROM artists WHERE canonical_name = 'Jared Leto'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '73GCZlf6pPNlMRnLuPkqIx', 'verified', 1.0
FROM artists WHERE canonical_name = 'Blood on the Dance Floor'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '23zg3TcAtWQy7J6upgbUnj', 'verified', 1.0
FROM artists WHERE canonical_name = 'Usher'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0c173mlxpT3dSFRgMO8XPh', 'verified', 1.0
FROM artists WHERE canonical_name = 'Big Sean'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5X7jkPoZ3jZMUYdV8zq7Bj', 'verified', 1.0
FROM artists WHERE canonical_name = 'Bow Wow'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6sFIWsNpZYqfjUpaCgueju', 'verified', 1.0
FROM artists WHERE canonical_name = 'Hedley'
ON CONFLICT (artist_id, platform) DO NOTHING;
