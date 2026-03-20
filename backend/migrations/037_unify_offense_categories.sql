-- Unify offense_category enum to superset of all crate definitions
-- Adds: animal_cruelty, financial_crimes, drug_offenses, violent_crimes, harassment, plagiarism
-- (certified_creeper was already added in migration 023)

DO $$
BEGIN
    -- Add missing enum values if they don't already exist
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'animal_cruelty' AND enumtypid = 'offense_category'::regtype) THEN
        ALTER TYPE offense_category ADD VALUE 'animal_cruelty';
    END IF;
END$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'financial_crimes' AND enumtypid = 'offense_category'::regtype) THEN
        ALTER TYPE offense_category ADD VALUE 'financial_crimes';
    END IF;
END$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'drug_offenses' AND enumtypid = 'offense_category'::regtype) THEN
        ALTER TYPE offense_category ADD VALUE 'drug_offenses';
    END IF;
END$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'violent_crimes' AND enumtypid = 'offense_category'::regtype) THEN
        ALTER TYPE offense_category ADD VALUE 'violent_crimes';
    END IF;
END$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'harassment' AND enumtypid = 'offense_category'::regtype) THEN
        ALTER TYPE offense_category ADD VALUE 'harassment';
    END IF;
END$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'plagiarism' AND enumtypid = 'offense_category'::regtype) THEN
        ALTER TYPE offense_category ADD VALUE 'plagiarism';
    END IF;
END$$;
