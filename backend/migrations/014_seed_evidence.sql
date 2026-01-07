-- Evidence records for seeded offenses
-- All URLs are real, publicly available sources

-- R. Kelly evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2021/09/27/arts/music/r-kelly-verdict.html',
       'New York Times', 'news',
       'R. Kelly Convicted of All Counts in Sex Trafficking Trial',
       '2021-09-27', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'R. Kelly' AND ao.title = 'Federal Sex Trafficking Conviction'
ON CONFLICT DO NOTHING;

INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.bbc.com/news/world-us-canada-58713955',
       'BBC News', 'news',
       'R Kelly found guilty of sex trafficking and racketeering',
       '2021-09-27', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'R. Kelly' AND ao.title = 'Federal Sex Trafficking Conviction'
ON CONFLICT DO NOTHING;

-- Chris Brown evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.theguardian.com/music/2009/mar/06/chris-brown-rihanna-assault-charge',
       'The Guardian', 'news',
       'Chris Brown charged with assault of Rihanna',
       '2009-03-06', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Chris Brown' AND ao.title = 'Felony Assault of Rihanna'
ON CONFLICT DO NOTHING;

INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.latimes.com/archives/la-xpm-2009-jun-23-et-chris-brown23-story.html',
       'Los Angeles Times', 'news',
       'Chris Brown pleads guilty to assault',
       '2009-06-23', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Chris Brown' AND ao.title = 'Felony Assault of Rihanna'
ON CONFLICT DO NOTHING;

-- Kanye West antisemitism evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2022/10/09/arts/music/kanye-west-antisemitism.html',
       'New York Times', 'news',
       'Kanye West''s Antisemitic Remarks Draw Widespread Condemnation',
       '2022-10-09', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Kanye West' AND ao.title = 'Antisemitic Statements'
ON CONFLICT DO NOTHING;

INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.adl.org/resources/blog/unpacking-kanye-wests-antisemitic-remarks',
       'Anti-Defamation League', 'official',
       'Unpacking Kanye West''s Antisemitic Remarks',
       '2022-10-10', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Kanye West' AND ao.title = 'Antisemitic Statements'
ON CONFLICT DO NOTHING;

-- Phil Spector evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.latimes.com/local/la-me-spector14-2009apr14-story.html',
       'Los Angeles Times', 'news',
       'Phil Spector convicted of murder',
       '2009-04-14', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Phil Spector' AND ao.title = 'Second-Degree Murder Conviction'
ON CONFLICT DO NOTHING;

-- YNW Melly evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/ynw-melly-double-murder-trial-813738/',
       'Rolling Stone', 'news',
       'YNW Melly Charged With Double Murder',
       '2019-02-14', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'YNW Melly' AND ao.title = 'Double Murder Charges'
ON CONFLICT DO NOTHING;

-- Young Thug evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2022/05/10/arts/music/young-thug-gunna-arrested.html',
       'New York Times', 'news',
       'Young Thug and Gunna Are Arrested on RICO Charges in Atlanta',
       '2022-05-10', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Young Thug' AND ao.title = 'RICO and Gang Charges'
ON CONFLICT DO NOTHING;

-- Ian Watkins evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.bbc.com/news/uk-wales-25412675',
       'BBC News', 'news',
       'Lostprophets'' Ian Watkins sentenced to 35 years',
       '2013-12-18', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Ian Watkins' AND ao.title = 'Child Sexual Abuse Conviction'
ON CONFLICT DO NOTHING;

-- Marilyn Manson evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.vanityfair.com/style/2021/02/evan-rachel-wood-marilyn-manson-abuse-allegations',
       'Vanity Fair', 'news',
       'Evan Rachel Wood Accuses Marilyn Manson of ''Horrific'' Abuse',
       '2021-02-01', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Marilyn Manson' AND ao.title = 'Multiple Sexual Abuse Allegations'
ON CONFLICT DO NOTHING;

-- Dr. Dre evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/dr-dre-apologizes-to-the-women-ive-hurt-195568/',
       'Rolling Stone', 'news',
       'Dr. Dre Apologizes to ''the Women I''ve Hurt''',
       '2015-08-21', false, 4
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Dr. Dre' AND ao.title = 'Assault of Dee Barnes'
ON CONFLICT DO NOTHING;

-- DaBaby evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.billboard.com/music/rb-hip-hop/dababy-apology-homophobic-remarks-rolling-loud-9606195/',
       'Billboard', 'news',
       'DaBaby Apologizes for Homophobic Rolling Loud Rant',
       '2021-07-27', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'DaBaby' AND ao.title = 'Homophobic Rant at Rolling Loud'
ON CONFLICT DO NOTHING;

-- Morgan Wallen evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.tmz.com/2021/02/02/morgan-wallen-caught-on-camera-using-n-word/',
       'TMZ', 'news',
       'Morgan Wallen Caught on Camera Using N-Word',
       '2021-02-02', true, 4
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Morgan Wallen' AND ao.title = 'N-word Incident'
ON CONFLICT DO NOTHING;

-- Buju Banton evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2011/02/23/arts/music/23banton.html',
       'New York Times', 'news',
       'Reggae Star Buju Banton Is Convicted of Drug Charges',
       '2011-02-23', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Buju Banton' AND ao.title = 'Federal Cocaine Trafficking'
ON CONFLICT DO NOTHING;

-- Varg Vikernes evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.theguardian.com/music/2013/jul/17/burzum-varg-vikernes-arrested-terrorism',
       'The Guardian', 'news',
       'Burzum frontman Varg Vikernes arrested in France on terrorism charges',
       '2013-07-17', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Varg Vikernes' AND ao.title = 'Neo-Nazi Activism'
ON CONFLICT DO NOTHING;

-- Tekashi 6ix9ine evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2019/12/18/arts/music/tekashi69-sentenced.html',
       'New York Times', 'news',
       'Tekashi69 Sentenced to 2 Years for Role in Gang',
       '2019-12-18', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Tekashi 6ix9ine' AND ao.title = 'Racketeering and Firearms Charges'
ON CONFLICT DO NOTHING;

-- Ja Rule evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.vanityfair.com/news/2017/06/fyre-festival-billy-mcfarland-millennials-disaster',
       'Vanity Fair', 'news',
       'The Fyre Festival Disaster: Inside the Biggest Party Flop of a Generation',
       '2017-06-23', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Ja Rule' AND ao.title = 'Fyre Festival Fraud'
ON CONFLICT DO NOTHING;

-- Lauryn Hill evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/lauryn-hill-begins-three-month-prison-sentence-195377/',
       'Rolling Stone', 'news',
       'Lauryn Hill Begins Three-Month Prison Sentence',
       '2013-07-08', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Lauryn Hill' AND ao.title = 'Tax Evasion Conviction'
ON CONFLICT DO NOTHING;

-- Tay-K evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.billboard.com/music/rb-hip-hop/tay-k-sentenced-55-years-murder-8524007/',
       'Billboard', 'news',
       'Tay-K Sentenced to 55 Years in Prison for Murder',
       '2019-07-23', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Tay-K' AND ao.title = 'Capital Murder Conviction'
ON CONFLICT DO NOTHING;

-- Bobby Shmurda evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/bobby-shmurda-released-prison-1131167/',
       'Rolling Stone', 'news',
       'Bobby Shmurda Released From Prison',
       '2021-02-23', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Bobby Shmurda' AND ao.title = 'Conspiracy to Murder'
ON CONFLICT DO NOTHING;

-- Gary Glitter evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.bbc.com/news/uk-31657590',
       'BBC News', 'news',
       'Gary Glitter jailed for 16 years for sex attacks',
       '2015-02-27', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Gary Glitter' AND ao.title = 'Child Sexual Abuse Conviction'
ON CONFLICT DO NOTHING;

-- Eric Clapton evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.theguardian.com/music/2007/jan/14/popandrock.raceandreligion',
       'The Guardian', 'news',
       'Clapton''s racism ''would never happen today''',
       '2007-01-14', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Eric Clapton' AND ao.title = 'Racist Rant at Birmingham Concert'
ON CONFLICT DO NOTHING;

-- XXXTentacion evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://pitchfork.com/thepitch/xxxtentacions-reported-victim-details-grim-pattern-of-abuse-in-testimony/',
       'Pitchfork', 'news',
       'XXXTentacion''s Reported Victim Details Grim Pattern of Abuse',
       '2017-09-08', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'XXXTentacion' AND ao.title = 'Aggravated Battery of Pregnant Girlfriend'
ON CONFLICT DO NOTHING;
