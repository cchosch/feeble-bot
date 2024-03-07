-- Your SQL goes here

CREATE TABLE users (
    id VARCHAR PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    "password" TEXT NOT NULL,
    email TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    verified_email BOOL NOT NULL,
    banned BOOL NOT NULL,
    kicked_until TIMESTAMPTZ NOT NULL,
    flags BIGINT NOT NULL
);

CREATE INDEX users_name
    ON users (username);
CREATE INDEX users_email
    ON users (email);
CREATE INDEX users_created
    ON users (created_at);