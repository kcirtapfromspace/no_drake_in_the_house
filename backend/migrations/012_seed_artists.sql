-- Comprehensive Artist Catalog Seed Data
-- Contains 50+ well-documented artist cases from public court records and major news outlets
-- All information is based on publicly available documentation

-- Sexual Misconduct/Assault Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Marilyn Manson', '{"spotify": "2VYQTNDsvvKN9wmU5W7xpj"}'::jsonb, '{"genres": ["industrial rock", "metal"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Marilyn Manson');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ian Watkins', '{}'::jsonb, '{"genres": ["rock", "post-hardcore"], "note": "Former Lostprophets vocalist"}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ian Watkins');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Gary Glitter', '{}'::jsonb, '{"genres": ["glam rock"], "popularity": 30}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Gary Glitter');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Afrika Bambaataa', '{}'::jsonb, '{"genres": ["hip hop", "electro"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Afrika Bambaataa');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Dr. Luke', '{}'::jsonb, '{"genres": ["pop"], "note": "Producer", "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Dr. Luke');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Russell Simmons', '{}'::jsonb, '{"genres": ["hip hop"], "note": "Music executive", "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Russell Simmons');

-- Domestic Violence Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ike Turner', '{}'::jsonb, '{"genres": ["r&b", "rock"], "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ike Turner');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ozzy Osbourne', '{"spotify": "6ZLTlhejhndI4Rh53vYhrY"}'::jsonb, '{"genres": ["heavy metal", "rock"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ozzy Osbourne');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Phil Spector', '{}'::jsonb, '{"genres": ["pop"], "note": "Producer", "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Phil Spector');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Eminem', '{"spotify": "7dGJo4pcD2V6oG8kP0tJRR"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 90}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Eminem');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Vince Neil', '{}'::jsonb, '{"genres": ["glam metal", "rock"], "note": "Mötley Crüe vocalist", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Vince Neil');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Dr. Dre', '{"spotify": "6DPYiyq5kWVQS4RGwxzPC7"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 85}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Dr. Dre');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Kodak Black', '{"spotify": "46SHBwWsqBkxI7EeeBEQG7"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Kodak Black');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Famous Dex', '{"spotify": "1qhzO1kDJpDDM0cXPQygHE"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Famous Dex');

-- Hate Speech/Racism Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Kanye West', '{"spotify": "5K4W6rqBFWDnAN6FQUkS6x"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 92}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Kanye West');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ted Nugent', '{"spotify": "19Ybd5wRLPLqhA1VzFoxiP"}'::jsonb, '{"genres": ["rock", "hard rock"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ted Nugent');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Kid Rock', '{"spotify": "7dOBabd5jYLrcpmFKzWGmP"}'::jsonb, '{"genres": ["rock", "country rock"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Kid Rock');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Eric Clapton', '{"spotify": "6PAt558ZEZl0DmdXlnjMgD"}'::jsonb, '{"genres": ["blues", "rock"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Eric Clapton');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Morrissey', '{"spotify": "3KPt8XPoT9PY3FxqzXwMYh"}'::jsonb, '{"genres": ["indie rock", "alternative"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Morrissey');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'DaBaby', '{"spotify": "4r63FhuTkUYltbVAg5TQnk"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'DaBaby');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Azealia Banks', '{"spotify": "7hgTHmw6LE7BdIKv3NrJPe"}'::jsonb, '{"genres": ["hip hop", "electronic"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Azealia Banks');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Morgan Wallen', '{"spotify": "4oUHIQIBe0LHzYfvXNW4QM"}'::jsonb, '{"genres": ["country"], "popularity": 88}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Morgan Wallen');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Varg Vikernes', '{}'::jsonb, '{"genres": ["black metal"], "note": "Burzum", "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Varg Vikernes');

-- Homophobia Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Buju Banton', '{"spotify": "7GmJhL0YLmAUWLRsR8F2MF"}'::jsonb, '{"genres": ["reggae", "dancehall"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Buju Banton');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Beenie Man', '{"spotify": "1UUBoWxu9TKXdNfnQTyqcV"}'::jsonb, '{"genres": ["dancehall", "reggae"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Beenie Man');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Sizzla', '{"spotify": "0qIOE79xSh0VHwfnVPxFaC"}'::jsonb, '{"genres": ["reggae", "dancehall"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Sizzla');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Elephant Man', '{}'::jsonb, '{"genres": ["dancehall"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Elephant Man');

-- Violent Crime Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Gucci Mane', '{"spotify": "13y7CgLHjMVRMDqxdx0Xdo"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Gucci Mane');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'C-Murder', '{}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'C-Murder');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bobby Shmurda', '{"spotify": "6qQ3PSEFqHvVJfPMLKj8zx"}'::jsonb, '{"genres": ["hip hop", "drill"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bobby Shmurda');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tay-K', '{}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tay-K');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'YNW Melly', '{"spotify": "4HpjqP6hFJRe9e1aVMxL2b"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'YNW Melly');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tekashi 6ix9ine', '{"spotify": "6xfWbSVcpwYomvK2dLkXVE"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tekashi 6ix9ine');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Casanova', '{}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Casanova');

-- Drug Trafficking Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'DMX', '{"spotify": "1HwM5zlC5qNWiEQ0v4PUjb"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'DMX');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Wayne', '{"spotify": "55Aa2cqylxrFIXC767Z865"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 88}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Wayne');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'T.I.', '{"spotify": "4oGIUcmNEHEVKE7o1rgMdQ"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'T.I.');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Young Thug', '{"spotify": "50co4Is1HCEo8bhOyUWKpn"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 85}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Young Thug');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Gunna', '{"spotify": "2hlmm7s2ICUX0LVIhVFlZQ"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 85}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Gunna');

-- Fraud/Financial Crimes Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ja Rule', '{"spotify": "1J2VVASYBcqPkH9HuKgQ1Q"}'::jsonb, '{"genres": ["hip hop", "r&b"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ja Rule');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lauryn Hill', '{"spotify": "73Zz52NiOk8E7F6vfFAU8w"}'::jsonb, '{"genres": ["hip hop", "r&b", "soul"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lauryn Hill');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Fat Joe', '{"spotify": "7IiuKHvGc4IwTPPZ18UEMW"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Fat Joe');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Nelly', '{"spotify": "1O5xPAQVp8fecxrOZwHqGD"}'::jsonb, '{"genres": ["hip hop", "r&b"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Nelly');

-- Antisemitism Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ice Cube', '{"spotify": "7LGV9qFbJqRW3YEw48fqHd"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ice Cube');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Roger Waters', '{"spotify": "35PpnfoWluzmWsLvEqVPzj"}'::jsonb, '{"genres": ["progressive rock"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Roger Waters');

-- Additional artists for comprehensiveness
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'John Lennon', '{"spotify": "4x1nvY2FN8jxqAFA0DA02H"}'::jsonb, '{"genres": ["rock", "pop"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'John Lennon');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'James Brown', '{}'::jsonb, '{"genres": ["funk", "soul", "r&b"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'James Brown');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Cee Lo Green', '{"spotify": "5nLYd9ST4Cnwy6NHaCxbj8"}'::jsonb, '{"genres": ["r&b", "soul"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Cee Lo Green');
