-- Add migration script here
CREATE TABLE update_history (
    'date' DATETIME NOT NULL,
    'name' VARCHAR(256) NOT NULL,
    'old_version' VARCHAR(256) NOT NULL,
    'updated_to' VARCHAR(256) NOT NULL,
    PRIMARY KEY ('date') 
);