-- Add migration script here
ALTER TABLE programs ADD current_version_last_updated DATETIME NOT NULL DEFAULT '1970-01-01 00:00:00';
ALTER TABLE programs ADD latest_version_last_updated DATETIME NOT NULL DEFAULT '1970-01-01 00:00:00';