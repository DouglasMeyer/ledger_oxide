ALTER TABLE projected_entries ADD COLUMN date DATE;

UPDATE projected_entries SET date = CURRENT_DATE WHERE date IS NULL;

ALTER TABLE projected_entries ALTER COLUMN date SET NOT NULL;
