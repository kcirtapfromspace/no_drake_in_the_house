-- Offense records for seeded artists
-- All based on publicly documented court records and major news outlets

-- R. Kelly offenses (already seeded in migration 010)
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'egregious'::offense_severity,
       'Federal Sex Trafficking Conviction',
       'Convicted on 9 counts of sex trafficking and racketeering. Sentenced to 30 years in federal prison after decades of allegations from multiple victims.',
       '2021-09-27', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'R. Kelly'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'egregious'::offense_severity,
       'Child Pornography Conviction',
       'Convicted on federal child pornography charges in Chicago trial. Multiple counts involving minors.',
       '2022-09-14', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'R. Kelly'
ON CONFLICT DO NOTHING;

-- Chris Brown offenses (already seeded in migration 010)
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'severe'::offense_severity,
       'Felony Assault of Rihanna',
       'Pleaded guilty to felony assault of then-girlfriend Rihanna the night before the 2009 Grammy Awards. Police photos showed extensive facial injuries.',
       '2009-02-08', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Chris Brown'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Multiple Assault Allegations',
       'Numerous assault allegations including incidents at clubs in NYC and Las Vegas, restraining orders filed by multiple women.',
       '2017-05-01', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Chris Brown'
ON CONFLICT DO NOTHING;

-- XXXTentacion offenses (already seeded)
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'severe'::offense_severity,
       'Aggravated Battery of Pregnant Girlfriend',
       'Charged with aggravated battery of pregnant girlfriend, domestic battery by strangulation, false imprisonment, and witness tampering. Detailed testimony from victim describing extensive abuse.',
       '2016-10-06', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'XXXTentacion'
ON CONFLICT DO NOTHING;

-- Marilyn Manson offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'severe'::offense_severity,
       'Multiple Sexual Abuse Allegations',
       'Multiple women including actress Evan Rachel Wood alleged physical, sexual, and emotional abuse. FBI investigation opened. Sued by multiple accusers.',
       '2021-02-01', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Marilyn Manson'
ON CONFLICT DO NOTHING;

-- Ian Watkins offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'egregious'::offense_severity,
       'Child Sexual Abuse Conviction',
       'Sentenced to 35 years in prison for multiple child sexual offenses including attempted rape of a baby. One of the most severe sentences in Welsh legal history.',
       '2013-12-18', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Ian Watkins'
ON CONFLICT DO NOTHING;

-- Gary Glitter offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'child_abuse'::offense_category, 'egregious'::offense_severity,
       'Child Sexual Abuse Conviction',
       'Multiple convictions for child sexual abuse in UK and Vietnam. Currently serving 16-year prison sentence in the UK.',
       '2015-02-27', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Gary Glitter'
ON CONFLICT DO NOTHING;

-- Kanye West offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'antisemitism'::offense_category, 'severe'::offense_severity,
       'Antisemitic Statements',
       'Made multiple antisemitic statements including "death con 3 on Jewish people" on social media. Lost major brand deals with Adidas, Gap, and others.',
       '2022-10-08', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Kanye West'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'hate_speech'::offense_category, 'severe'::offense_severity,
       'Hitler Praise on Alex Jones Show',
       'Praised Adolf Hitler on Alex Jones show, stating "I see good things about Hitler" and "I love Nazis." Resulted in Twitter/X account suspension.',
       '2022-12-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Kanye West'
ON CONFLICT DO NOTHING;

-- Dr. Dre offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'severe'::offense_severity,
       'Assault of Dee Barnes',
       'Pleaded no contest to misdemeanor battery charges for assault of journalist Dee Barnes at a record release party. Barnes filed civil suit.',
       '1991-01-27', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Dr. Dre'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'moderate'::offense_severity,
       'Allegations by Michel''le',
       'Ex-girlfriend and singer Michel''le detailed years of physical abuse in interviews and documentary, including broken nose and black eyes.',
       '1990-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Dr. Dre'
ON CONFLICT DO NOTHING;

-- Phil Spector offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Second-Degree Murder Conviction',
       'Convicted of second-degree murder of actress Lana Clarkson. Sentenced to 19 years to life. Died in prison in 2021.',
       '2003-02-03', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Phil Spector'
ON CONFLICT DO NOTHING;

-- YNW Melly offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Double Murder Charges',
       'Charged with two counts of first-degree murder of friends/group members YNW Sakchaser and YNW Juvy. Allegedly staged the scene to look like a drive-by shooting.',
       '2019-02-13', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'YNW Melly'
ON CONFLICT DO NOTHING;

-- Young Thug offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'severe'::offense_severity,
       'RICO and Gang Charges',
       'Arrested on RICO charges, accused of being a founder of Young Slime Life gang. Charged with conspiracy to violate RICO act, participation in street gang activity.',
       '2022-05-09', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Young Thug'
ON CONFLICT DO NOTHING;

-- Gunna offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'severe'::offense_severity,
       'RICO Charges and Plea Deal',
       'Arrested on RICO charges alongside Young Thug. Took plea deal, released after serving time, pleaded guilty to one count of conspiracy.',
       '2022-05-09', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Gunna'
ON CONFLICT DO NOTHING;

-- Tekashi 6ix9ine offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Racketeering and Firearms Charges',
       'Pleaded guilty to nine charges including racketeering, firearms offenses, and drug trafficking. Became federal witness, testified against Nine Trey Gangsta Bloods.',
       '2018-11-18', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tekashi 6ix9ine'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'sexual_misconduct'::offense_category, 'moderate'::offense_severity,
       'Sexual Performance by a Child',
       'Pleaded guilty to use of a child in a sexual performance. Placed on probation and registered as sex offender.',
       '2015-10-20', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tekashi 6ix9ine'
ON CONFLICT DO NOTHING;

-- DaBaby offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'moderate'::offense_severity,
       'Homophobic Rant at Rolling Loud',
       'Made homophobic statements during Rolling Loud performance, spreading misinformation about HIV/AIDS. Multiple festivals cancelled his appearances.',
       '2021-07-25', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'DaBaby'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'moderate'::offense_severity,
       'Fatal Shooting in Walmart',
       'Shot and killed a man during an altercation at a Walmart. Claimed self-defense, charges reduced and eventually dropped.',
       '2018-11-05', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'DaBaby'
ON CONFLICT DO NOTHING;

-- Morgan Wallen offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'racism'::offense_category, 'moderate'::offense_severity,
       'N-word Incident',
       'Video surfaced of him using racial slur outside his home. Radio stations dropped his music, awards shows uninvited him, record label suspended him.',
       '2021-02-02', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Morgan Wallen'
ON CONFLICT DO NOTHING;

-- Buju Banton offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'homophobia'::offense_category, 'severe'::offense_severity,
       'Boom Bye Bye - Murder Lyrics',
       'Song "Boom Bye Bye" explicitly calls for violence against gay men. Faced widespread protests and concert cancellations for decades.',
       '1992-06-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Buju Banton'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'severe'::offense_severity,
       'Federal Cocaine Trafficking',
       'Convicted of conspiracy to possess cocaine with intent to distribute. Served 7 years in federal prison.',
       '2011-02-22', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Buju Banton'
ON CONFLICT DO NOTHING;

-- Varg Vikernes offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Murder of Euronymous',
       'Convicted of murdering Mayhem guitarist Euronymous with 23 stab wounds. Served 15 years in Norwegian prison.',
       '1993-08-10', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Varg Vikernes'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'hate_speech'::offense_category, 'egregious'::offense_severity,
       'Neo-Nazi Activism',
       'Openly promotes neo-Nazi ideology, white supremacy. Arrested in France on terrorism charges. Blog posts promoting racial hatred.',
       '2013-07-16', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Varg Vikernes'
ON CONFLICT DO NOTHING;

-- Ja Rule offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'fraud'::offense_category, 'moderate'::offense_severity,
       'Fyre Festival Fraud',
       'Co-organized Fyre Festival, which defrauded ticket holders. Billy McFarland convicted of fraud. Ja Rule faced civil suits.',
       '2017-04-28', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Ja Rule'
ON CONFLICT DO NOTHING;

-- Lauryn Hill offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'fraud'::offense_category, 'moderate'::offense_severity,
       'Tax Evasion Conviction',
       'Pleaded guilty to failing to file tax returns on $1.8 million income. Served 3 months in federal prison.',
       '2013-06-26', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Lauryn Hill'
ON CONFLICT DO NOTHING;

-- DMX offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'drug_trafficking'::offense_category, 'severe'::offense_severity,
       'Tax Evasion and Drug Charges',
       'Convicted of tax fraud, sentenced to one year in prison. Multiple drug possession arrests throughout career.',
       '2018-03-29', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'DMX'
ON CONFLICT DO NOTHING;

INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'animal_abuse'::offense_category, 'moderate'::offense_severity,
       'Animal Cruelty Charges',
       'Charged with animal cruelty after authorities found 12 neglected pit bulls at his Arizona home. Some dogs had to be euthanized.',
       '2008-01-01', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'DMX'
ON CONFLICT DO NOTHING;

-- Tay-K offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Capital Murder Conviction',
       'Convicted of murder at age 19. Sentenced to 55 years in prison for involvement in home invasion robbery that resulted in death.',
       '2019-07-23', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Tay-K'
ON CONFLICT DO NOTHING;

-- C-Murder offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'egregious'::offense_severity,
       'Second-Degree Murder Conviction',
       'Convicted of second-degree murder of 16-year-old Steve Thomas at nightclub. Sentenced to life in prison.',
       '2002-01-12', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'C-Murder'
ON CONFLICT DO NOTHING;

-- Bobby Shmurda offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'violent_crime'::offense_category, 'severe'::offense_severity,
       'Conspiracy to Murder',
       'Pleaded guilty to conspiracy and weapons possession charges. Part of indictment involving GS9 gang members. Served 6 years in prison.',
       '2014-12-17', true, true, true, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Bobby Shmurda'
ON CONFLICT DO NOTHING;

-- Ice Cube offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'antisemitism'::offense_category, 'moderate'::offense_severity,
       'Antisemitic Social Media Posts',
       'Shared antisemitic imagery and conspiracy theories on social media. Faced criticism but defended posts.',
       '2020-06-10', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Ice Cube'
ON CONFLICT DO NOTHING;

-- Roger Waters offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'antisemitism'::offense_category, 'moderate'::offense_severity,
       'Antisemitic Concert Imagery',
       'Used Star of David and pig imagery in concerts. German authorities investigated for incitement. Accused by Anti-Defamation League.',
       '2023-05-24', false, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Roger Waters'
ON CONFLICT DO NOTHING;

-- Ted Nugent offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'hate_speech'::offense_category, 'severe'::offense_severity,
       'Racist and Homophobic Statements',
       'Made numerous racist statements about Obama, called him "subhuman mongrel." Made repeated homophobic statements in interviews.',
       '2014-01-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Ted Nugent'
ON CONFLICT DO NOTHING;

-- Morrissey offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'racism'::offense_category, 'severe'::offense_severity,
       'Far-Right Political Statements',
       'Made statements supporting far-right For Britain party, criticized immigration, made controversial statements about Chinese people.',
       '2019-05-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Morrissey'
ON CONFLICT DO NOTHING;

-- John Lennon offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'moderate'::offense_severity,
       'Self-Admitted Domestic Violence',
       'Lennon admitted in interviews to hitting women in his past relationships. Discussed violence toward first wife Cynthia in Playboy interview.',
       '1980-09-01', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'John Lennon'
ON CONFLICT DO NOTHING;

-- Ozzy Osbourne offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'domestic_violence'::offense_category, 'moderate'::offense_severity,
       'Attempted Murder of Sharon',
       'Attempted to strangle wife Sharon while heavily intoxicated in 1989. Sharon has spoken publicly about the incident.',
       '1989-09-02', true, true, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Ozzy Osbourne'
ON CONFLICT DO NOTHING;

-- Eric Clapton offenses
INSERT INTO artist_offenses (artist_id, category, severity, title, description, incident_date, arrested, charged, convicted, status)
SELECT id, 'racism'::offense_category, 'moderate'::offense_severity,
       'Racist Rant at Birmingham Concert',
       'Made racist statements during 1976 Birmingham concert, calling for "foreigners" to leave UK, using racial slurs.',
       '1976-08-05', false, false, false, 'verified'::evidence_status
FROM artists WHERE canonical_name = 'Eric Clapton'
ON CONFLICT DO NOTHING;
