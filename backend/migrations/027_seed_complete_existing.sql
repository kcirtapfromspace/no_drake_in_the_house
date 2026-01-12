-- Migration: 027_seed_complete_existing.sql
-- Phase 1: Complete existing artists with missing offenses, evidence, and platform IDs

-- =============================================
-- PART 1: ADD MISSING OFFENSES FOR EXISTING ARTISTS
-- =============================================

-- Afrika Bambaataa - child abuse allegations
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'egregious'::offense_severity,
       'Multiple Child Sexual Abuse Allegations',
       'Multiple men came forward alleging sexual abuse by Bambaataa during their youth in the 1980s. Universal Zulu Nation initially defended him before later expelling him.',
       '2016-04-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Afrika Bambaataa'
ON CONFLICT DO NOTHING;

-- Dr. Luke - sexual abuse allegations
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Sexual Abuse Allegations by Kesha',
       'Singer Kesha alleged Dr. Luke drugged and sexually assaulted her over a period of years. Lengthy legal battle ensued. Dr. Luke won defamation lawsuit in 2023.',
       '2014-10-14', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Dr. Luke'
ON CONFLICT DO NOTHING;

-- Russell Simmons - sexual misconduct
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Multiple Sexual Assault Allegations',
       'Over 20 women accused Russell Simmons of sexual misconduct ranging from harassment to rape. He stepped down from his companies. Documentary "On the Record" detailed allegations.',
       '2017-11-30', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Russell Simmons'
ON CONFLICT DO NOTHING;

-- Ike Turner - domestic violence
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'egregious'::offense_severity,
       'Decades of Abuse Against Tina Turner',
       'Tina Turner documented years of severe physical and emotional abuse in her autobiography and interviews. She fled in 1976 with only 36 cents.',
       '1976-07-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Ike Turner'
ON CONFLICT DO NOTHING;

-- Vince Neil - vehicular manslaughter
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Vehicular Manslaughter',
       'While driving drunk, crashed car killing Hanoi Rocks drummer Razzle and injuring two others. Served 15 days of 30-day sentence. Paid $2.6 million in restitution.',
       '1984-12-08', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Vince Neil'
ON CONFLICT DO NOTHING;

-- Kodak Black - sexual assault
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Sexual Assault Charge',
       'Charged with first-degree criminal sexual conduct in South Carolina after allegedly assaulting a woman in a hotel room. Pleaded guilty to first-degree assault.',
       '2016-02-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Kodak Black'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'severe'::offense_severity,
       'Federal Weapons and Drug Charges',
       'Arrested at Rolling Loud festival attempting to bring weapons into the country. Pleaded guilty to federal weapons charges. Sentenced to 46 months, later commuted by Trump.',
       '2019-05-11', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Kodak Black'
ON CONFLICT DO NOTHING;

-- Famous Dex - domestic violence
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'severe'::offense_severity,
       'Video of Domestic Violence',
       'Viral video showed Famous Dex physically assaulting his girlfriend. He later apologized and claimed to be seeking help for anger issues.',
       '2016-08-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Famous Dex'
ON CONFLICT DO NOTHING;

-- Kid Rock - assault
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Waffle House Assault',
       'Arrested for assaulting a man at a Waffle House in Atlanta. Pleaded no contest to misdemeanor battery.',
       '2007-10-21', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Kid Rock'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'hate_speech'::offense_category, 'moderate'::offense_severity,
       'Confederate Flag Controversy',
       'Long history of using Confederate flag imagery in concerts and merchandise. Has defended the flag and used racial slurs in interviews.',
       '2015-07-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Kid Rock'
ON CONFLICT DO NOTHING;

-- Azealia Banks - multiple controversies
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'hate_speech'::offense_category, 'moderate'::offense_severity,
       'Multiple Racist and Homophobic Attacks',
       'Numerous incidents using racial slurs against Zayn Malik, homophobic slurs against Perez Hilton, and other celebrities. Banned from Twitter multiple times.',
       '2016-05-10', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Azealia Banks'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'minor'::offense_severity,
       'Assault of Security Guard',
       'Arrested and charged with misdemeanor assault for biting a security guard''s breast at a nightclub.',
       '2015-12-16', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Azealia Banks'
ON CONFLICT DO NOTHING;

-- Gucci Mane - multiple violent incidents
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Murder Charge - Self Defense',
       'Shot and killed a man during an attempted robbery at his home. Charges dropped due to insufficient evidence and self-defense claim.',
       '2005-05-10', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Gucci Mane'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Assault of Fan with Bottle',
       'Threw fan off stage and hit a soldier in the head with a champagne bottle at a military event. Pleaded guilty to aggravated assault.',
       '2011-04-05', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Gucci Mane'
ON CONFLICT DO NOTHING;

-- Casanova - RICO charges
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'RICO Conspiracy and Gun Charges',
       'Arrested on federal RICO charges related to Untouchable Gorilla Stone Nation gang. Pleaded guilty to racketeering conspiracy and firearms charges.',
       '2020-12-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Casanova'
ON CONFLICT DO NOTHING;

-- Lil Wayne - weapons charges
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Gun Possession Charge',
       'Pleaded guilty to attempted criminal possession of a weapon in New York. Served 8 months at Rikers Island.',
       '2010-03-08', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Lil Wayne'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Federal Weapons Charge Miami',
       'Charged with federal gun possession after authorities found loaded gold-plated handgun on his private jet. Pardoned by President Trump.',
       '2019-12-23', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Lil Wayne'
ON CONFLICT DO NOTHING;

-- T.I. - weapons charges
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'severe'::offense_severity,
       'Federal Weapons Charges',
       'Arrested attempting to purchase machine guns, silencers, and other firearms as a convicted felon. Served 7 months in federal prison.',
       '2007-10-13', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'T.I.'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Sexual Abuse Allegations',
       'Multiple women accused T.I. and wife Tiny of drugging and sexually assaulting them. Under investigation in multiple states. Denied all allegations.',
       '2021-01-28', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'T.I.'
ON CONFLICT DO NOTHING;

-- Beenie Man - homophobia
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'severe'::offense_severity,
       'Murder Music Lyrics',
       'Multiple songs including "Damn" and "Bad Man Chi Chi Man" call for violence against LGBTQ+ people. Faced widespread protests and concert cancellations.',
       '2004-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Beenie Man'
ON CONFLICT DO NOTHING;

-- Sizzla - homophobia
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'severe'::offense_severity,
       'Anti-LGBTQ Lyrics and Statements',
       'Multiple songs calling for violence against LGBTQ+ individuals. Concert cancellations worldwide. Named in Stop Murder Music campaign.',
       '2004-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Sizzla'
ON CONFLICT DO NOTHING;

-- Elephant Man - homophobia
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'severe'::offense_severity,
       'Anti-LGBTQ Dancehall Lyrics',
       'Songs including "We Nuh Like Gay" and others promote violence against LGBTQ+ people. Part of dancehall "murder music" controversy.',
       '2004-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Elephant Man'
ON CONFLICT DO NOTHING;

-- Fat Joe - tax evasion
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'fraud'::offense_category, 'moderate'::offense_severity,
       'Tax Evasion Conviction',
       'Pleaded guilty to tax evasion for failing to pay taxes on over $3 million in income. Served 4 months in federal prison.',
       '2013-06-24', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Fat Joe'
ON CONFLICT DO NOTHING;

-- Nelly - drug and sexual assault
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'Drug and Gun Possession',
       'Arrested when tour bus was found to contain heroin, marijuana, and a loaded gun. Pleaded guilty to misdemeanor drug possession.',
       '2012-10-11', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Nelly'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'moderate'::offense_severity,
       'Sexual Assault Allegation',
       'Woman accused Nelly of raping her on his tour bus. Charges dropped after accuser declined to testify. Settled civil lawsuit.',
       '2017-10-07', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Nelly'
ON CONFLICT DO NOTHING;

-- James Brown - domestic violence
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'severe'::offense_severity,
       'Multiple Domestic Violence Arrests',
       'Arrested multiple times for domestic violence against wives. Third wife Adrienne Rodriguez called police at least 13 times. Pleaded no contest to domestic violence in 2004.',
       '1988-04-03', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'James Brown'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'moderate'::offense_severity,
       'PCP Incident and Police Chase',
       'Led police on high-speed chase while high on PCP, fired shotgun at police. Sentenced to 6 years in prison, served 2 years.',
       '1988-09-24', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'James Brown'
ON CONFLICT DO NOTHING;

-- Cee Lo Green - sexual assault
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'moderate'::offense_severity,
       'Ecstasy Assault Case',
       'Charged with furnishing MDMA to a woman at dinner. Originally investigated for sexual assault but insufficient evidence. Pleaded no contest to drug charge. Made controversial rape comments on Twitter.',
       '2012-07-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Cee Lo Green'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 2: ADD EVIDENCE FOR NEW OFFENSES
-- =============================================

-- Afrika Bambaataa evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2016/05/10/arts/music/afrika-bambaataa-zulu-nation-sexual-abuse.html',
       'New York Times', 'news',
       'Afrika Bambaataa Asked to Step Down Amid Sexual Abuse Allegations',
       '2016-05-10', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Afrika Bambaataa' AND ao.title = 'Multiple Child Sexual Abuse Allegations'
ON CONFLICT DO NOTHING;

-- Dr. Luke evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2023/06/22/arts/music/kesha-dr-luke-defamation-trial.html',
       'New York Times', 'news',
       'Kesha Loses Defamation Trial to Dr. Luke',
       '2023-06-22', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Dr. Luke' AND ao.title = 'Sexual Abuse Allegations by Kesha'
ON CONFLICT DO NOTHING;

-- Russell Simmons evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2017/12/13/arts/russell-simmons-rape.html',
       'New York Times', 'news',
       'Russell Simmons Is Accused of Rape by 3 Women',
       '2017-12-13', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Russell Simmons' AND ao.title = 'Multiple Sexual Assault Allegations'
ON CONFLICT DO NOTHING;

-- Ike Turner evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/tina-turner-ike-turner-abuse-1191374/',
       'Rolling Stone', 'news',
       'Tina Turner''s Most Harrowing Ike Turner Abuse Revelations',
       '2021-03-27', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Ike Turner' AND ao.title = 'Decades of Abuse Against Tina Turner'
ON CONFLICT DO NOTHING;

-- Vince Neil evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.latimes.com/archives/la-xpm-1985-04-13-me-14191-story.html',
       'Los Angeles Times', 'news',
       'Vince Neil Sentenced in Death of Hanoi Rocks Drummer',
       '1985-04-13', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Vince Neil' AND ao.title = 'Vehicular Manslaughter'
ON CONFLICT DO NOTHING;

-- Kodak Black evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.billboard.com/music/rb-hip-hop/kodak-black-pleads-guilty-assault-sexual-assault-case-8520186/',
       'Billboard', 'news',
       'Kodak Black Pleads Guilty to Assault in Sexual Assault Case',
       '2019-04-25', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Kodak Black' AND ao.title = 'Sexual Assault Charge'
ON CONFLICT DO NOTHING;

INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2021/01/20/us/politics/trump-pardons.html',
       'New York Times', 'news',
       'Trump Pardons Kodak Black and Lil Wayne',
       '2021-01-20', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Kodak Black' AND ao.title = 'Federal Weapons and Drug Charges'
ON CONFLICT DO NOTHING;

-- Famous Dex evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.hotnewhiphop.com/famous-dex-apologizes-for-beating-his-girlfriend-news.23313.html',
       'HotNewHipHop', 'news',
       'Famous Dex Apologizes For Beating His Girlfriend',
       '2016-08-22', false, 4
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Famous Dex' AND ao.title = 'Video of Domestic Violence'
ON CONFLICT DO NOTHING;

-- Kid Rock evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/kid-rock-pleads-no-contest-in-waffle-house-assault-69666/',
       'Rolling Stone', 'news',
       'Kid Rock Pleads No Contest in Waffle House Assault',
       '2008-04-21', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Kid Rock' AND ao.title = 'Waffle House Assault'
ON CONFLICT DO NOTHING;

-- Azealia Banks evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.theguardian.com/music/2016/may/11/azealia-banks-twitter-account-suspended-zayn-malik',
       'The Guardian', 'news',
       'Azealia Banks'' Twitter Account Suspended After Zayn Malik Row',
       '2016-05-11', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Azealia Banks' AND ao.title = 'Multiple Racist and Homophobic Attacks'
ON CONFLICT DO NOTHING;

-- Gucci Mane evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2006/01/18/arts/music/gucci-mane-and-the-violence-that-hip-hop-culture-embraces.html',
       'New York Times', 'news',
       'Gucci Mane and the Violence That Hip-Hop Culture Embraces',
       '2006-01-18', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Gucci Mane' AND ao.title = 'Murder Charge - Self Defense'
ON CONFLICT DO NOTHING;

-- Casanova evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2020/12/03/nyregion/casanova-rapper-gang-indictment.html',
       'New York Times', 'news',
       'Rapper Casanova Indicted on Racketeering Charges',
       '2020-12-03', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Casanova' AND ao.title = 'RICO Conspiracy and Gun Charges'
ON CONFLICT DO NOTHING;

-- Lil Wayne evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2010/03/09/arts/music/09wayne.html',
       'New York Times', 'news',
       'Lil Wayne Gets One-Year Sentence on Gun Charge',
       '2010-03-09', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Lil Wayne' AND ao.title = 'Gun Possession Charge'
ON CONFLICT DO NOTHING;

-- T.I. evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.nytimes.com/2007/10/16/arts/music/16cnd-rapper.html',
       'New York Times', 'news',
       'Rapper T.I. Arrested on Federal Gun Charges',
       '2007-10-16', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'T.I.' AND ao.title = 'Federal Weapons Charges'
ON CONFLICT DO NOTHING;

-- Beenie Man evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.theguardian.com/world/2004/aug/09/gayrights.arts',
       'The Guardian', 'news',
       'Gay Groups Boycott Reggae Artists Over ''Murder Music''',
       '2004-08-09', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Beenie Man' AND ao.title = 'Murder Music Lyrics'
ON CONFLICT DO NOTHING;

-- Sizzla evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.theguardian.com/world/2004/aug/09/gayrights.arts',
       'The Guardian', 'news',
       'Gay Groups Boycott Reggae Artists Over ''Murder Music''',
       '2004-08-09', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Sizzla' AND ao.title = 'Anti-LGBTQ Lyrics and Statements'
ON CONFLICT DO NOTHING;

-- Fat Joe evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.billboard.com/music/rb-hip-hop/fat-joe-sentenced-four-months-prison-tax-evasion-5614219/',
       'Billboard', 'news',
       'Fat Joe Sentenced to Four Months in Prison for Tax Evasion',
       '2013-06-24', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Fat Joe' AND ao.title = 'Tax Evasion Conviction'
ON CONFLICT DO NOTHING;

-- James Brown evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.rollingstone.com/music/music-news/the-troubled-history-of-james-brown-204203/',
       'Rolling Stone', 'news',
       'The Troubled History of James Brown',
       '2007-01-05', false, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'James Brown' AND ao.title = 'Multiple Domestic Violence Arrests'
ON CONFLICT DO NOTHING;

-- Cee Lo Green evidence
INSERT INTO offense_evidence (offense_id, url, source_name, source_type, title, published_date, is_primary_source, credibility_score)
SELECT ao.id,
       'https://www.billboard.com/music/rb-hip-hop/cee-lo-green-pleads-no-contest-furnishing-ecstasy-6236543/',
       'Billboard', 'news',
       'Cee Lo Green Pleads No Contest to Furnishing Ecstasy',
       '2014-08-29', true, 5
FROM artist_offenses ao
JOIN artists a ON ao.artist_id = a.id
WHERE a.canonical_name = 'Cee Lo Green' AND ao.title = 'Ecstasy Assault Case'
ON CONFLICT DO NOTHING;

-- =============================================
-- PART 3: ADD PLATFORM IDS FOR EXISTING ARTISTS
-- =============================================

-- Artists missing Spotify IDs
INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7GlBOeep6PqTfFi59PTUUN', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ian Watkins'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2dIgFjalVxs4ThymZ67YCE', 'verified', 1.0
FROM artists WHERE canonical_name = 'Gary Glitter'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5mWeyDs2Id1XnvpZJJDmzK', 'verified', 1.0
FROM artists WHERE canonical_name = 'Afrika Bambaataa'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0RDjeNEXkYpLwcLnehANS1', 'verified', 1.0
FROM artists WHERE canonical_name = 'Russell Simmons'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2Cm4Bi8VNaFJcF9KcxoqYk', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ike Turner'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4KykqFM6fRrwLuZItYcJRV', 'verified', 1.0
FROM artists WHERE canonical_name = 'Phil Spector'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1W9mhYwDgslppjFfxJFGGK', 'verified', 1.0
FROM artists WHERE canonical_name = 'Vince Neil'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0bBYxKkhgkWGcfM5qKvl6T', 'verified', 1.0
FROM artists WHERE canonical_name = 'C-Murder'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5gNxPYYhMnJY5xrFqQq6GU', 'verified', 1.0
FROM artists WHERE canonical_name = 'Tay-K'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4BoHIgPnSO9mVOYFJZV08C', 'verified', 1.0
FROM artists WHERE canonical_name = 'Casanova'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4IVAbR2w4JJNJDDRFP3E83', 'verified', 1.0
FROM artists WHERE canonical_name = 'Elephant Man'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6n75TL0Cxg4viYqUvZNOUn', 'verified', 1.0
FROM artists WHERE canonical_name = 'Varg Vikernes'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2tIP7SsRs7vjIcLrU85W8J', 'verified', 1.0
FROM artists WHERE canonical_name = 'James Brown'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5oLNtHGThYppYPfFXH1FWT', 'verified', 1.0
FROM artists WHERE canonical_name = 'Dr. Luke'
ON CONFLICT (artist_id, platform) DO NOTHING;

-- Add platform IDs for artists already in database with external_ids but missing artist_platform_ids entry
INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2VYQTNDsvvKN9wmU5W7xpj', 'verified', 1.0
FROM artists WHERE canonical_name = 'Marilyn Manson'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6ZLTlhejhndI4Rh53vYhrY', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ozzy Osbourne'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7dGJo4pcD2V6oG8kP0tJRR', 'verified', 1.0
FROM artists WHERE canonical_name = 'Eminem'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6DPYiyq5kWVQS4RGwxzPC7', 'verified', 1.0
FROM artists WHERE canonical_name = 'Dr. Dre'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '46SHBwWsqBkxI7EeeBEQG7', 'verified', 1.0
FROM artists WHERE canonical_name = 'Kodak Black'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1qhzO1kDJpDDM0cXPQygHE', 'verified', 1.0
FROM artists WHERE canonical_name = 'Famous Dex'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5K4W6rqBFWDnAN6FQUkS6x', 'verified', 1.0
FROM artists WHERE canonical_name = 'Kanye West'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '19Ybd5wRLPLqhA1VzFoxiP', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ted Nugent'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7dOBabd5jYLrcpmFKzWGmP', 'verified', 1.0
FROM artists WHERE canonical_name = 'Kid Rock'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6PAt558ZEZl0DmdXlnjMgD', 'verified', 1.0
FROM artists WHERE canonical_name = 'Eric Clapton'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '3KPt8XPoT9PY3FxqzXwMYh', 'verified', 1.0
FROM artists WHERE canonical_name = 'Morrissey'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4r63FhuTkUYltbVAg5TQnk', 'verified', 1.0
FROM artists WHERE canonical_name = 'DaBaby'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7hgTHmw6LE7BdIKv3NrJPe', 'verified', 1.0
FROM artists WHERE canonical_name = 'Azealia Banks'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4oUHIQIBe0LHzYfvXNW4QM', 'verified', 1.0
FROM artists WHERE canonical_name = 'Morgan Wallen'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7GmJhL0YLmAUWLRsR8F2MF', 'verified', 1.0
FROM artists WHERE canonical_name = 'Buju Banton'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1UUBoWxu9TKXdNfnQTyqcV', 'verified', 1.0
FROM artists WHERE canonical_name = 'Beenie Man'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '0qIOE79xSh0VHwfnVPxFaC', 'verified', 1.0
FROM artists WHERE canonical_name = 'Sizzla'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '13y7CgLHjMVRMDqxdx0Xdo', 'verified', 1.0
FROM artists WHERE canonical_name = 'Gucci Mane'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6qQ3PSEFqHvVJfPMLKj8zx', 'verified', 1.0
FROM artists WHERE canonical_name = 'Bobby Shmurda'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4HpjqP6hFJRe9e1aVMxL2b', 'verified', 1.0
FROM artists WHERE canonical_name = 'YNW Melly'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '6xfWbSVcpwYomvK2dLkXVE', 'verified', 1.0
FROM artists WHERE canonical_name = 'Tekashi 6ix9ine'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1HwM5zlC5qNWiEQ0v4PUjb', 'verified', 1.0
FROM artists WHERE canonical_name = 'DMX'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '55Aa2cqylxrFIXC767Z865', 'verified', 1.0
FROM artists WHERE canonical_name = 'Lil Wayne'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4oGIUcmNEHEVKE7o1rgMdQ', 'verified', 1.0
FROM artists WHERE canonical_name = 'T.I.'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '50co4Is1HCEo8bhOyUWKpn', 'verified', 1.0
FROM artists WHERE canonical_name = 'Young Thug'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '2hlmm7s2ICUX0LVIhVFlZQ', 'verified', 1.0
FROM artists WHERE canonical_name = 'Gunna'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1J2VVASYBcqPkH9HuKgQ1Q', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ja Rule'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '73Zz52NiOk8E7F6vfFAU8w', 'verified', 1.0
FROM artists WHERE canonical_name = 'Lauryn Hill'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7IiuKHvGc4IwTPPZ18UEMW', 'verified', 1.0
FROM artists WHERE canonical_name = 'Fat Joe'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '1O5xPAQVp8fecxrOZwHqGD', 'verified', 1.0
FROM artists WHERE canonical_name = 'Nelly'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '7LGV9qFbJqRW3YEw48fqHd', 'verified', 1.0
FROM artists WHERE canonical_name = 'Ice Cube'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '35PpnfoWluzmWsLvEqVPzj', 'verified', 1.0
FROM artists WHERE canonical_name = 'Roger Waters'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '4x1nvY2FN8jxqAFA0DA02H', 'verified', 1.0
FROM artists WHERE canonical_name = 'John Lennon'
ON CONFLICT (artist_id, platform) DO NOTHING;

INSERT INTO artist_platform_ids (artist_id, platform, platform_id, verification_status, confidence_score)
SELECT id, 'spotify', '5nLYd9ST4Cnwy6NHaCxbj8', 'verified', 1.0
FROM artists WHERE canonical_name = 'Cee Lo Green'
ON CONFLICT (artist_id, platform) DO NOTHING;
