-- Your SQL goes here
CREATE TABLE sessions (
    sess_id TEXT primary key,
    sess_cookie TEXT not null,
    expiry timestamptz not null,
    uid TEXT,
    "data" json
);

CREATE INDEX sessions_uid
ON sessions (uid);
