-- Your SQL goes here
CREATE TABLE controlled_account (
    id VARCHAR PRIMARY KEY,
    discord_id VARCHAR NOT NULL,
    username VARCHAR NOT NULL,
    token VARCHAR NOT NULL,
    created_by VARCHAR NOT NULL
);

CREATE TABLE account_mapping (
    id VARCHAR PRIMARY KEY,
    mapped_discord_id VARCHAR NOT NULL,
    controlled_discord_id VARCHAR NOT NULL,
    controlled_internal_id VARCHAR NOT NULL,
    controlled_username VARCHAR NOT NULL
);
