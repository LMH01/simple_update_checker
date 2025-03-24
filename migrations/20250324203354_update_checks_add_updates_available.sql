-- Add migration script here
ALTER TABLE update_checks
RENAME TO update_check_history;

ALTER TABLE update_check_history
ADD updates_available INTEGER;

ALTER TABLE update_check_history
ADD programs VARCHAR(4096);