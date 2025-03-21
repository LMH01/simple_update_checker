-- Add migration script here
CREATE TABLE programs (
    'name' VARCHAR(256) NOT NULL,
    latest_version VARCHAR(256) NOT NULL,
    'provider' VARCHAR(256) NOT NULL,
    PRIMARY KEY ('name')
);

CREATE TABLE github_programs (
    'name' VARCHAR(256) NOT NULL,
    repository VARCHAR(256) NOT NULL UNIQUE,
    FOREIGN KEY ('name') REFERENCES programs('name')
);