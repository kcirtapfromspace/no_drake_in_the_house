-- Migration: 028_seed_artists_batch1_violence.sql
-- Batch 1: Violent Crime, Murder, and Assault Artists (~80 artists)

-- =============================================
-- PART 1: INSERT NEW ARTISTS - Violent Crime Category
-- =============================================

-- Murder/Manslaughter Convictions
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Snoop Dogg', '{"spotify": "7JVmFSIFLGHX0ZAb2xTPhO"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 90}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Snoop Dogg');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Sid Vicious', '{}'::jsonb, '{"genres": ["punk rock"], "note": "Sex Pistols bassist", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Sid Vicious');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Don King', '{}'::jsonb, '{"genres": ["music promoter"], "note": "Boxing promoter with music industry ties", "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Don King');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Leadbelly', '{}'::jsonb, '{"genres": ["blues", "folk"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Leadbelly');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Jim Gordon', '{}'::jsonb, '{"genres": ["rock"], "note": "Session drummer", "popularity": 30}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Jim Gordon');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Spade Cooley', '{}'::jsonb, '{"genres": ["western swing", "country"], "popularity": 25}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Spade Cooley');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bertrand Cantat', '{}'::jsonb, '{"genres": ["rock"], "note": "Noir Désir vocalist", "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bertrand Cantat');

-- Gang-Related Violence - Drill/Trap Artists
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'King Von', '{"spotify": "5DWOioAQUgqwIGlGwRbKjX"}'::jsonb, '{"genres": ["drill", "hip hop"], "popularity": 78}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'King Von');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'FBG Duck', '{"spotify": "2vZqnuBCZR8LJahvTFgRdm"}'::jsonb, '{"genres": ["drill", "hip hop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'FBG Duck');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Durk', '{"spotify": "3hcs9uc56yIGFCSy9leWe7"}'::jsonb, '{"genres": ["drill", "hip hop", "trap"], "popularity": 88}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Durk');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Pooh Shiesty', '{"spotify": "36tEcHJOPx8QpvSH4ORdnl"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Pooh Shiesty');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Ralo', '{"spotify": "4c3PXJqQhVYMwpkPKmLNeD"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Ralo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Boskoe100', '{}'::jsonb, '{"genres": ["hip hop"], "note": "West Coast rapper", "popularity": 30}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Boskoe100');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Drakeo the Ruler', '{"spotify": "0fDk3eBApzJvJWVeGGQ6cR"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Drakeo the Ruler');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Foolio', '{"spotify": "0HXqNnBwMJJkVKArKtRb7x"}'::jsonb, '{"genres": ["hip hop", "drill"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Foolio');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'NBA YoungBoy', '{"spotify": "7wlFDEWiM5OoIAt8RSli8b"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 92}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'NBA YoungBoy');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Loaded', '{"spotify": "52FX0HKwJKaVGCTI2A6jrR"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Loaded');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Quando Rondo', '{"spotify": "1fctva5IxBQ6PmDITbMtUx"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Quando Rondo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'EST Gee', '{"spotify": "1Ld9C6OctzBOdwn2r6CPHm"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'EST Gee');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Blueface', '{"spotify": "4dgmqjxzx7dqxUVyuDNgYU"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Blueface');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Moneybagg Yo', '{"spotify": "3VDKXCjOp3A7SjW4J0wvZY"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Moneybagg Yo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Offset', '{"spotify": "0UE1JKmDVSPmL45mh3eG4U"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 82}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Offset');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Quavo', '{"spotify": "0VRj0yCOv2FXJNP47XQnx5"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Quavo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Max B', '{"spotify": "5w8NzIBZOe0HJZyJB0fMlL"}'::jsonb, '{"genres": ["hip hop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Max B');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Shyne', '{"spotify": "0rEmCyHBYxZX6NFxjCRpqA"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Shyne');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Boosie', '{"spotify": "36E7oYfz3LLRto6yCpf85B"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Boosie');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Soulja Slim', '{"spotify": "1LQqHXxK1KwKZjAVQkOsVv"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Soulja Slim');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Mac Dre', '{"spotify": "6DgNxpO8MJ5lB8U3aHMXN8"}'::jsonb, '{"genres": ["hip hop", "hyphy"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Mac Dre');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Trouble', '{"spotify": "3mNwNBHhb4q2q9jNPvJyjt"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Trouble');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'PnB Rock', '{"spotify": "2Tz1DTzVJ5Gyh8ZwVr6ekU"}'::jsonb, '{"genres": ["hip hop", "r&b"], "popularity": 68}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'PnB Rock');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Mo3', '{"spotify": "4dVxI4t7d5H7PbOjq1MCXB"}'::jsonb, '{"genres": ["hip hop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Mo3');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Nipsey Hussle', '{"spotify": "3DP4I0MaLJg1L5SXtQAhh5"}'::jsonb, '{"genres": ["hip hop", "west coast rap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Nipsey Hussle');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Pop Smoke', '{"spotify": "0eDvMgVFoNV3TpwtrVCoTj"}'::jsonb, '{"genres": ["drill", "hip hop"], "popularity": 85}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Pop Smoke');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Young Dolph', '{"spotify": "3FOhL6AKdGgGdMPxGDwh1F"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Young Dolph');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Takeoff', '{"spotify": "4NMpLvXjSjVBMIlBgmCNxv"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 78}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Takeoff');

-- Assault Convictions
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'A$AP Rocky', '{"spotify": "13ubrt8QOOCPljQ2FL1Kca"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 85}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'A$AP Rocky');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Waka Flocka Flame', '{"spotify": "6f4XkbvYlXMH0QgVRzW0sM"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 68}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Waka Flocka Flame');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Busta Rhymes', '{"spotify": "1YfEcTuGvBQ8xSD1f53UnK"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Busta Rhymes');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'The Game', '{"spotify": "0NbfKEOTQCcwd6o7wSDOHI"}'::jsonb, '{"genres": ["hip hop", "west coast rap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'The Game');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Fabolous', '{"spotify": "04hh2JSSTAGKlRiKh7J6t8"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Fabolous');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Mystikal', '{"spotify": "7JjUjJ1V7B1MphJDDq1lqa"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Mystikal');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Sticky Fingaz', '{"spotify": "0L0sSqMoNlS87OAeVP8FWY"}'::jsonb, '{"genres": ["hip hop", "rap"], "note": "Onyx member", "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Sticky Fingaz');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Safaree', '{"spotify": "6gNTYr8Cs0HcQ0Y5lmZj2I"}'::jsonb, '{"genres": ["hip hop", "r&b"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Safaree');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Young Buck', '{"spotify": "3IPCWrSiN5X9GlKv7mRGe8"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Young Buck');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Plies', '{"spotify": "1M1wjqWReYj0VUaGZWkP52"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Plies');

-- Armed Robbery / Carjacking
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Remy Ma', '{"spotify": "2x5rQtJHPzPPAhpJD93yJN"}'::jsonb, '{"genres": ["hip hop", "rap"], "popularity": 68}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Remy Ma');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'G Perico', '{"spotify": "4LwM4G6KvhdGLYZ5pHvXXw"}'::jsonb, '{"genres": ["hip hop", "west coast rap"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'G Perico');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Zay Osama', '{"spotify": "6RmgaqVDNXEJwJgRdNVLHm"}'::jsonb, '{"genres": ["drill", "hip hop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Zay Osama');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bandman Kevo', '{"spotify": "6EpU2RTFj0PREIotbnpzNT"}'::jsonb, '{"genres": ["hip hop", "drill"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bandman Kevo');

-- UK Drill / Violence
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Digga D', '{"spotify": "1cOBPS1cCwMDJiHxKHyugm"}'::jsonb, '{"genres": ["uk drill", "hip hop"], "popularity": 65}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Digga D');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'AM', '{"spotify": "7BoIK0zcTflJ1ycMlggCCQ"}'::jsonb, '{"genres": ["uk drill", "hip hop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'AM');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Skengdo', '{"spotify": "1bAKRjHcPnFUKD6L2pN1mB"}'::jsonb, '{"genres": ["uk drill", "hip hop"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Skengdo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Incognito', '{"spotify": "1Jlkj9QKPdJCJ2o8nG5A0y"}'::jsonb, '{"genres": ["uk drill", "hip hop"], "note": "Zone 2 member", "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Incognito');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'CB', '{"spotify": "5Fbs2CaUQqH5tTNmXCGT3M"}'::jsonb, '{"genres": ["uk drill", "hip hop"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'CB');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'LD', '{"spotify": "3ZvG8H1bDu5SdEX8IZZQ8T"}'::jsonb, '{"genres": ["uk drill", "hip hop"], "note": "67 member", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'LD');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Dimzy', '{"spotify": "0wVHyFfVH9s5RkdRZYW1qD"}'::jsonb, '{"genres": ["uk drill", "hip hop"], "note": "67 member", "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Dimzy');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'SJ', '{"spotify": "2iL22p4FrNzwgk8kB0sLJH"}'::jsonb, '{"genres": ["uk drill", "hip hop"], "note": "OFB member", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'SJ');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Headie One', '{"spotify": "6UCQYrcJ6wab6gnQ89OJFh"}'::jsonb, '{"genres": ["uk drill", "hip hop"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Headie One');

-- Rock/Metal Violence
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Burzum', '{"spotify": "6n75TL0Cxg4viYqUvZNOUn"}'::jsonb, '{"genres": ["black metal", "ambient"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Burzum');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Mayhem', '{"spotify": "5LjpXY4lHJLV3vJq5pNDvF"}'::jsonb, '{"genres": ["black metal"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Mayhem');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Emperor', '{"spotify": "7K0HBbWHlKdZ23OJGmvwsO"}'::jsonb, '{"genres": ["black metal"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Emperor');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Gorgoroth', '{"spotify": "1RMJOxR6GjWJrBhCUniIEf"}'::jsonb, '{"genres": ["black metal"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Gorgoroth');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Dissection', '{"spotify": "3f6IhDM5yDixXrF3YKvnnC"}'::jsonb, '{"genres": ["black metal", "melodic death metal"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Dissection');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tim Lambesis', '{}'::jsonb, '{"genres": ["metalcore", "metal"], "note": "As I Lay Dying vocalist", "popularity": 35}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tim Lambesis');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Randy Blythe', '{"spotify": "2krKdOlzPk8LWKVbmMCkJL"}'::jsonb, '{"genres": ["metal", "groove metal"], "note": "Lamb of God vocalist", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Randy Blythe');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Nikki Sixx', '{"spotify": "3LqKPxI6f1M8ZC0HnXhLue"}'::jsonb, '{"genres": ["glam metal", "rock"], "note": "Mötley Crüe bassist", "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Nikki Sixx');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tommy Lee', '{"spotify": "6YuT6MfY3a3BW1tEylJq3Y"}'::jsonb, '{"genres": ["glam metal", "rock"], "note": "Mötley Crüe drummer", "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tommy Lee');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Axl Rose', '{"spotify": "6wN4NSuMJTIzFfvXE0AaLB"}'::jsonb, '{"genres": ["rock", "hard rock"], "note": "Guns N'' Roses vocalist", "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Axl Rose');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Motley Crue', '{"spotify": "0cc6vw3VN8YlIcvr1v7tBL"}'::jsonb, '{"genres": ["glam metal", "rock"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Motley Crue');

-- Other Violent Incidents
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'OJ da Juiceman', '{"spotify": "4mJxNxwFZNZ2LbLpYPKc8t"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 50}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'OJ da Juiceman');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Webbie', '{"spotify": "0L2W1B0j9fNAzk4TW59V5G"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Webbie');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Bone Crusher', '{"spotify": "2C3WT3kPnmBXqgQVdBHjah"}'::jsonb, '{"genres": ["hip hop", "crunk"], "popularity": 45}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Bone Crusher');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Project Pat', '{"spotify": "7mPq2rSAiYSyKMMPdQ9lPt"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 55}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Project Pat');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Juicy J', '{"spotify": "5gCRApTajqwbnHHPbr2Fpi"}'::jsonb, '{"genres": ["hip hop", "southern rap"], "popularity": 75}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Juicy J');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Three 6 Mafia', '{"spotify": "2e70TuBrMPWrBZBYzVzC4j"}'::jsonb, '{"genres": ["hip hop", "southern rap", "horrorcore"], "popularity": 70}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Three 6 Mafia');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Chief Keef', '{"spotify": "15iVAtD3s3FsQR4w1v6M0P"}'::jsonb, '{"genres": ["drill", "hip hop"], "popularity": 80}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Chief Keef');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Lil Reese', '{"spotify": "6rNDeUFZmSCyI3o9K3PqtP"}'::jsonb, '{"genres": ["drill", "hip hop"], "popularity": 60}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Lil Reese');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'G Herbo', '{"spotify": "5QdEbQJ3ylBnc3gsIASAT5"}'::jsonb, '{"genres": ["drill", "hip hop"], "popularity": 78}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'G Herbo');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Polo G', '{"spotify": "6AgTAQt8XS6jRWi4sX7w49"}'::jsonb, '{"genres": ["hip hop", "drill"], "popularity": 85}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Polo G');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT '21 Savage', '{"spotify": "1URnnhqYAYcrqrcwql10ft"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 88}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = '21 Savage');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Tee Grizzley', '{"spotify": "6QfVYLgR49p7jMcJuC2Bao"}'::jsonb, '{"genres": ["hip hop", "trap"], "popularity": 72}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Tee Grizzley');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Suge Knight', '{}'::jsonb, '{"genres": ["hip hop"], "note": "Death Row Records founder", "popularity": 40}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Suge Knight');

-- =============================================
-- PART 2: INSERT OFFENSES FOR NEW ARTISTS
-- =============================================

-- Snoop Dogg
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Murder Trial - Acquitted',
       'Charged with first-degree murder and being an accessory to murder for a 1993 drive-by shooting. His bodyguard McKinley Lee shot and killed Philip Woldemariam. Both acquitted in 1996 trial.',
       '1993-08-25', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Snoop Dogg'
ON CONFLICT DO NOTHING;

-- Sid Vicious
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder Charge - Nancy Spungen',
       'Charged with second-degree murder of girlfriend Nancy Spungen who was found stabbed at Chelsea Hotel. Died of heroin overdose before trial.',
       '1978-10-12', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Sid Vicious'
ON CONFLICT DO NOTHING;

-- Leadbelly
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Murder Conviction',
       'Convicted of murder and served time in Texas prison. Later convicted of assault with intent to murder in Louisiana. Pardoned by Louisiana governor.',
       '1918-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Leadbelly'
ON CONFLICT DO NOTHING;

-- Jim Gordon
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder of Mother',
       'Murdered his mother with a hammer and knife. Diagnosed with schizophrenia, found guilty but mentally ill. Serving 16 years to life.',
       '1983-06-03', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Jim Gordon'
ON CONFLICT DO NOTHING;

-- Spade Cooley
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder of Wife',
       'Brutally beat wife Ella Mae to death in front of daughter. Convicted of first-degree murder. Died shortly after parole hearing approval.',
       '1961-04-03', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Spade Cooley'
ON CONFLICT DO NOTHING;

-- Bertrand Cantat
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder of Marie Trintignant',
       'Beat actress and girlfriend Marie Trintignant to death in Lithuania hotel room. Convicted of voluntary manslaughter, served 4 years.',
       '2003-07-27', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Bertrand Cantat'
ON CONFLICT DO NOTHING;

-- King Von
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder Charges',
       'Charged with murder in 2014 shooting death. Acquitted. Associated with multiple violent incidents and gang feuds. Shot and killed in 2020.',
       '2014-05-29', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'King Von'
ON CONFLICT DO NOTHING;

-- Lil Durk
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Attempted Murder Charge',
       'Charged with attempted murder in connection with a shooting outside an Atlanta restaurant. Charges later dropped.',
       '2019-02-05', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Lil Durk'
ON CONFLICT DO NOTHING;

-- Pooh Shiesty
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Armed Robbery and Shooting',
       'Pleaded guilty to federal conspiracy to commit robbery charges. Shot a man during a sneaker deal that turned into a robbery. Sentenced to 63 months.',
       '2020-10-09', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Pooh Shiesty'
ON CONFLICT DO NOTHING;

-- NBA YoungBoy
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Kidnapping and Assault',
       'Arrested for kidnapping and assault. Multiple weapons charges. Drive-by shooting at his studio injured his girlfriend. Extensive legal history.',
       '2018-02-25', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'NBA YoungBoy'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'severe'::offense_severity,
       'Federal Gun and Drug Charges',
       'Arrested on federal firearms charges after fleeing from police. Previously acquitted of federal gun charges in 2022.',
       '2020-09-28', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'NBA YoungBoy'
ON CONFLICT DO NOTHING;

-- Lil Loaded
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Manslaughter Charge',
       'Charged with murder after friend Khalil Walker was shot and killed. Charge reduced to manslaughter. Died by suicide before trial.',
       '2020-10-25', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Lil Loaded'
ON CONFLICT DO NOTHING;

-- Quando Rondo
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Gang-Related Violence',
       'Present during altercation that resulted in King Von''s death. Multiple gang-related incidents and arrests. Charged with gang-related offenses.',
       '2020-11-06', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Quando Rondo'
ON CONFLICT DO NOTHING;

-- EST Gee
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Shooting Incident and Criminal History',
       'Shot multiple times in 2019 in retaliation shooting. Previous criminal convictions. Lyrics detail violent criminal activities.',
       '2019-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'EST Gee'
ON CONFLICT DO NOTHING;

-- Blueface
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Attempted Murder Charge',
       'Charged with attempted murder and assault with a deadly weapon after allegedly shooting at a man in Las Vegas. Multiple prior assault arrests.',
       '2022-11-15', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Blueface'
ON CONFLICT DO NOTHING;

-- A$AP Rocky
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Sweden Assault Conviction',
       'Convicted of assault in Sweden after a street fight. Held in Swedish jail for weeks. Sentenced to suspended sentence and fine.',
       '2019-06-30', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'A$AP Rocky'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Shooting Assault Charge',
       'Charged with two counts of assault with a semiautomatic firearm for allegedly shooting at former A$AP Mob member A$AP Relli.',
       '2021-11-06', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'A$AP Rocky'
ON CONFLICT DO NOTHING;

-- The Game
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Gun Threat at Basketball Game',
       'Arrested for making criminal threats after pulling gun on an off-duty officer at a basketball game. Pleaded no contest.',
       '2007-02-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'The Game'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Sexual Assault Lawsuit',
       'Ordered to pay $7 million in sexual assault lawsuit. Woman alleged she was sexually assaulted during appearance on his VH1 dating show.',
       '2016-01-01', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'The Game'
ON CONFLICT DO NOTHING;

-- Fabolous
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'severe'::offense_severity,
       'Domestic Violence - Emily B',
       'Arrested for domestic violence after allegedly hitting girlfriend Emily B, knocking out two of her teeth, and threatening her father. Pleaded guilty to lesser charges.',
       '2018-03-28', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Fabolous'
ON CONFLICT DO NOTHING;

-- Mystikal
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'egregious'::offense_severity,
       'Sexual Battery Conviction',
       'Pleaded guilty to sexual battery, served 6 years in prison. Required to register as sex offender. Arrested again on rape charges in 2022.',
       '2003-01-15', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Mystikal'
ON CONFLICT DO NOTHING;

-- Remy Ma
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Shooting of Friend',
       'Shot friend Makeda Barnes-Joseph outside Manhattan nightclub over money dispute. Convicted of assault, served 6 years in prison.',
       '2007-07-14', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Remy Ma'
ON CONFLICT DO NOTHING;

-- Max B
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Armed Robbery and Murder',
       'Convicted of aggravated manslaughter, armed robbery, and kidnapping. Orchestrated robbery that resulted in murder. Originally sentenced to 75 years, reduced.',
       '2006-09-26', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Max B'
ON CONFLICT DO NOTHING;

-- Shyne
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Club New York Shooting',
       'Convicted of assault and reckless endangerment for shooting at NYC nightclub that injured three people. Served 10 years, deported to Belize.',
       '1999-12-27', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Shyne'
ON CONFLICT DO NOTHING;

-- Lil Boosie
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'First-Degree Murder Charge - Acquitted',
       'Charged with first-degree murder in 2009 killing. Acquitted. Previously convicted of drug charges, served 5 years.',
       '2010-09-22', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Lil Boosie'
ON CONFLICT DO NOTHING;

-- Chief Keef
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Multiple Gun and Assault Charges',
       'Numerous arrests for weapons possession, assault, and probation violations. Discharged gun at a firing range while on probation.',
       '2013-01-15', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Chief Keef'
ON CONFLICT DO NOTHING;

-- G Herbo
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'fraud'::offense_category, 'moderate'::offense_severity,
       'Wire Fraud Conspiracy',
       'Pleaded guilty to federal wire fraud for using stolen credit card info to fund luxury lifestyle. Sentenced to 3 years probation.',
       '2020-12-02', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'G Herbo'
ON CONFLICT DO NOTHING;

-- Polo G
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Battery and Resisting Arrest',
       'Arrested in Miami on multiple charges including battery of a police officer, resisting arrest, and criminal mischief.',
       '2021-06-12', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Polo G'
ON CONFLICT DO NOTHING;

-- Tee Grizzley
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Armed Robbery Conviction',
       'Convicted of armed robbery series at Michigan State University. Served 3 years. Manager/aunt killed in 2019 drive-by shooting.',
       '2014-06-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tee Grizzley'
ON CONFLICT DO NOTHING;

-- Suge Knight
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Voluntary Manslaughter',
       'Ran over and killed Terry Carter in Compton parking lot. Pleaded no contest to voluntary manslaughter. Sentenced to 28 years in prison.',
       '2015-01-29', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Suge Knight'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Assault History',
       'Long history of violent behavior. Assaulted Vanilla Ice on hotel balcony. Multiple assault charges throughout career. Suspected in 2Pac/Biggie murders.',
       '1995-01-01', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Suge Knight'
ON CONFLICT DO NOTHING;

-- Digga D
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Violent Disorder Conviction',
       'Convicted of violent disorder after group attack. Subject to Criminal Behaviour Order restricting lyrics. Multiple gang-related incidents.',
       '2018-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Digga D'
ON CONFLICT DO NOTHING;

-- Headie One
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Knife Possession Conviction',
       'Convicted of knife possession, sentenced to prison. Multiple stints in jail for weapons offenses.',
       '2019-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Headie One'
ON CONFLICT DO NOTHING;

-- SJ
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder Conviction',
       'Convicted of murder at age 16 in stabbing of rival gang member. Sentenced to life with minimum of 21 years. OFB (Original Farm Boys) member.',
       '2019-11-22', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'SJ'
ON CONFLICT DO NOTHING;

-- Tim Lambesis
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder-for-Hire Plot',
       'Arrested for hiring undercover officer to kill estranged wife. Pleaded guilty to solicitation of murder. Served 2 years of 6-year sentence.',
       '2013-05-07', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tim Lambesis'
ON CONFLICT DO NOTHING;

-- Randy Blythe
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Manslaughter Charge - Acquitted',
       'Charged with manslaughter in Czech Republic after fan died from injuries sustained when pushed off stage. Acquitted after trial.',
       '2012-06-28', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Randy Blythe'
ON CONFLICT DO NOTHING;

-- Tommy Lee
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'severe'::offense_severity,
       'Felony Spousal Battery - Pamela Anderson',
       'Kicked then-wife Pamela Anderson while she held their son. Pleaded no contest to felony spousal battery. Served 6 months in jail.',
       '1998-02-24', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tommy Lee'
ON CONFLICT DO NOTHING;

-- Axl Rose
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Inciting a Riot',
       'Dove into audience at St. Louis concert, punched fan, and abandoned show. Riot ensued causing 60 injuries. Arrested, pleaded guilty.',
       '1991-07-02', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Axl Rose'
ON CONFLICT DO NOTHING;

-- Project Pat
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Aggravated Robbery Conviction',
       'Convicted of aggravated robbery and carjacking. Served 4 years in prison. Brother of Juicy J, Three 6 Mafia member.',
       '2001-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Project Pat'
ON CONFLICT DO NOTHING;

-- Dissection
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder Conspiracy - Jon Nödtveidt',
       'Frontman Jon Nödtveidt convicted as accessory to murder of homosexual man in Gothenburg. Served 7 years. Committed suicide in 2006.',
       '1997-07-22', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Dissection'
ON CONFLICT DO NOTHING;

-- Emperor
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder and Church Burnings - Faust',
       'Drummer Bård "Faust" Eithun convicted of murdering homosexual man in Olympic Park. Also involved in church burnings. Served 9 years.',
       '1992-08-21', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Emperor'
ON CONFLICT DO NOTHING;

-- Mayhem
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder and Suicide - Inner Circle',
       'Vocalist Dead committed suicide. Euronymous photographed body, made necklaces from skull. Varg Vikernes murdered Euronymous. Multiple church arsons.',
       '1993-08-10', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Mayhem'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 3: INSERT EVIDENCE FOR NEW OFFENSES
-- =============================================

-- Snoop Dogg evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.latimes.com/archives/la-xpm-1996-02-21-mn-38399-story.html',
       'Los Angeles Times', 'news',
       'Snoop Doggy Dogg Acquitted of Murder',
       '1996-02-21', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Snoop Dogg' AND ao.title = 'Murder Trial - Acquitted'
ON CONFLICT DO NOTHING;

-- Sid Vicious evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/sid-vicious-dead-at-21-66135/',
       'Rolling Stone', 'news',
       'Sid Vicious Dead at 21',
       '1979-02-02', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Sid Vicious' AND ao.title = 'Murder Charge - Nancy Spungen'
ON CONFLICT DO NOTHING;

-- King Von evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.xxlmag.com/king-von-reportedly-shot-killed/',
       'XXL Magazine', 'news',
       'King Von Shot and Killed',
       '2020-11-06', false, 4
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'King Von' AND ao.title = 'Murder Charges'
ON CONFLICT DO NOTHING;

-- Pooh Shiesty evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.billboard.com/music/rb-hip-hop/pooh-shiesty-sentenced-63-months-federal-case-1235087431/',
       'Billboard', 'news',
       'Pooh Shiesty Sentenced to 63 Months in Federal Case',
       '2022-04-19', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Pooh Shiesty' AND ao.title = 'Armed Robbery and Shooting'
ON CONFLICT DO NOTHING;

-- A$AP Rocky evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2019/08/14/arts/music/asap-rocky-verdict-sweden.html',
       'New York Times', 'news',
       'A$AP Rocky Found Guilty of Assault in Sweden',
       '2019-08-14', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'A$AP Rocky' AND ao.title = 'Sweden Assault Conviction'
ON CONFLICT DO NOTHING;

-- Fabolous evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2018/03/28/nyregion/fabolous-rapper-arrested-domestic-violence.html',
       'New York Times', 'news',
       'Fabolous Arrested on Domestic Violence Charges',
       '2018-03-28', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Fabolous' AND ao.title = 'Domestic Violence - Emily B'
ON CONFLICT DO NOTHING;

-- Remy Ma evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2008/05/13/nyregion/13remy.html',
       'New York Times', 'news',
       'Rapper Remy Ma Gets 8-Year Sentence for Shooting',
       '2008-05-13', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Remy Ma' AND ao.title = 'Shooting of Friend'
ON CONFLICT DO NOTHING;

-- Suge Knight evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2018/09/20/us/suge-knight-sentenced.html',
       'New York Times', 'news',
       'Suge Knight Sentenced to 28 Years in Prison',
       '2018-09-20', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Suge Knight' AND ao.title = 'Voluntary Manslaughter'
ON CONFLICT DO NOTHING;

-- Tim Lambesis evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/as-i-lay-dying-singer-tim-lambesis-sentenced-to-six-years-in-prison-104825/',
       'Rolling Stone', 'news',
       'As I Lay Dying Singer Tim Lambesis Sentenced to Six Years in Prison',
       '2014-05-16', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Tim Lambesis' AND ao.title = 'Murder-for-Hire Plot'
ON CONFLICT DO NOTHING;

-- Tommy Lee evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/tommy-lee-sentenced-to-six-months-for-kicking-pamela-179892/',
       'Rolling Stone', 'news',
       'Tommy Lee Sentenced to Six Months for Kicking Pamela',
       '1998-05-21', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Tommy Lee' AND ao.title = 'Felony Spousal Battery - Pamela Anderson'
ON CONFLICT DO NOTHING;

-- Bertrand Cantat evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.theguardian.com/world/2004/mar/27/france.arts',
       'The Guardian', 'news',
       'Rock Star Gets Eight Years for Killing Actress',
       '2004-03-27', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Bertrand Cantat' AND ao.title = 'Murder of Marie Trintignant'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 4: INSERT PLATFORM IDS
-- =============================================

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7JVmFSIFLGHX0ZAb2xTPhO', 'verified', 1.0
FROM artists WHERE canonical_name = 'Snoop Dogg'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5DWOioAQUgqwIGlGwRbKjX', 'verified', 1.0
FROM artists WHERE canonical_name = 'King Von'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3hcs9uc56yIGFCSy9leWe7', 'verified', 1.0
FROM artists WHERE canonical_name = 'Lil Durk'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '36tEcHJOPx8QpvSH4ORdnl', 'verified', 1.0
FROM artists WHERE canonical_name = 'Pooh Shiesty'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7wlFDEWiM5OoIAt8RSli8b', 'verified', 1.0
FROM artists WHERE canonical_name = 'NBA YoungBoy'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4dgmqjxzx7dqxUVyuDNgYU', 'verified', 1.0
FROM artists WHERE canonical_name = 'Blueface'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0UE1JKmDVSPmL45mh3eG4U', 'verified', 1.0
FROM artists WHERE canonical_name = 'Offset'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0VRj0yCOv2FXJNP47XQnx5', 'verified', 1.0
FROM artists WHERE canonical_name = 'Quavo'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '13ubrt8QOOCPljQ2FL1Kca', 'verified', 1.0
FROM artists WHERE canonical_name = 'A$AP Rocky'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0NbfKEOTQCcwd6o7wSDOHI', 'verified', 1.0
FROM artists WHERE canonical_name = 'The Game'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '04hh2JSSTAGKlRiKh7J6t8', 'verified', 1.0
FROM artists WHERE canonical_name = 'Fabolous'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7JjUjJ1V7B1MphJDDq1lqa', 'verified', 1.0
FROM artists WHERE canonical_name = 'Mystikal'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2x5rQtJHPzPPAhpJD93yJN', 'verified', 1.0
FROM artists WHERE canonical_name = 'Remy Ma'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5w8NzIBZOe0HJZyJB0fMlL', 'verified', 1.0
FROM artists WHERE canonical_name = 'Max B'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '36E7oYfz3LLRto6yCpf85B', 'verified', 1.0
FROM artists WHERE canonical_name = 'Lil Boosie'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '15iVAtD3s3FsQR4w1v6M0P', 'verified', 1.0
FROM artists WHERE canonical_name = 'Chief Keef'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5QdEbQJ3ylBnc3gsIASAT5', 'verified', 1.0
FROM artists WHERE canonical_name = 'G Herbo'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6AgTAQt8XS6jRWi4sX7w49', 'verified', 1.0
FROM artists WHERE canonical_name = 'Polo G'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1URnnhqYAYcrqrcwql10ft', 'verified', 1.0
FROM artists WHERE canonical_name = '21 Savage'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6QfVYLgR49p7jMcJuC2Bao', 'verified', 1.0
FROM artists WHERE canonical_name = 'Tee Grizzley'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6UCQYrcJ6wab6gnQ89OJFh', 'verified', 1.0
FROM artists WHERE canonical_name = 'Headie One'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1cOBPS1cCwMDJiHxKHyugm', 'verified', 1.0
FROM artists WHERE canonical_name = 'Digga D'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6YuT6MfY3a3BW1tEylJq3Y', 'verified', 1.0
FROM artists WHERE canonical_name = 'Tommy Lee'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0cc6vw3VN8YlIcvr1v7tBL', 'verified', 1.0
FROM artists WHERE canonical_name = 'Motley Crue'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5LjpXY4lHJLV3vJq5pNDvF', 'verified', 1.0
FROM artists WHERE canonical_name = 'Mayhem'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3VDKXCjOp3A7SjW4J0wvZY', 'verified', 1.0
FROM artists WHERE canonical_name = 'Moneybagg Yo'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3DP4I0MaLJg1L5SXtQAhh5', 'verified', 1.0
FROM artists WHERE canonical_name = 'Nipsey Hussle'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0eDvMgVFoNV3TpwtrVCoTj', 'verified', 1.0
FROM artists WHERE canonical_name = 'Pop Smoke'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3FOhL6AKdGgGdMPxGDwh1F', 'verified', 1.0
FROM artists WHERE canonical_name = 'Young Dolph'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4NMpLvXjSjVBMIlBgmCNxv', 'verified', 1.0
FROM artists WHERE canonical_name = 'Takeoff'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5gCRApTajqwbnHHPbr2Fpi', 'verified', 1.0
FROM artists WHERE canonical_name = 'Juicy J'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2e70TuBrMPWrBZBYzVzC4j', 'verified', 1.0
FROM artists WHERE canonical_name = 'Three 6 Mafia'
ON CONFLICT (artist_id, platform) DO NOTHING;
