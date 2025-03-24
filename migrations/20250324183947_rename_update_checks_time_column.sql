-- Add migration script here
ALTER TABLE update_checks
RENAME COLUMN 'time' to 'date';