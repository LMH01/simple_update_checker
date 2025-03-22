-- Add migration script here
ALTER TABLE programs ADD current_version VARCHAR(256) NOT NULL DEFAULT '';
Update programs SET current_version = latest_version Where current_version = '';