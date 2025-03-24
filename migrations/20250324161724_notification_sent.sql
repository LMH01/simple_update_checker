-- Add migration script here
ALTER TABLE programs ADD notification_sent BOOLEAN NOT NULL DEFAULT FALSE;