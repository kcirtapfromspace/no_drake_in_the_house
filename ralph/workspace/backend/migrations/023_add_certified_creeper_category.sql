-- Add 'certified_creeper' to offense_category enum
-- For artists with inappropriate behavior with minors or grooming allegations

ALTER TYPE offense_category ADD VALUE IF NOT EXISTS 'certified_creeper' AFTER 'child_abuse';
