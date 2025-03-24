-- Add migration script here
CREATE TABLE update_history (
    'time' DATETIME NOT NULL,
    'name' VARCHAR(256) NOT NULL,
    'current_version' VARCHAR(256) NOT NULL,
    'updated_to' VARCHAR(256) NOT NULL,
    PRIMARY KEY ('time') 
);