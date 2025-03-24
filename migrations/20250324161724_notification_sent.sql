-- Add migration script here
ALTER TABLE programs ADD notification_sent BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE programs ADD notification_sent_on DATE;