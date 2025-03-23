-- Add migration script here
CREATE TABLE update_checks (
    'time' DATETIME NOT NULL,
    'type' VARCHAR(256) NOT NULL,
    PRIMARY KEY ('time')
);