-- Your SQL goes here
CREATE TABLE account_mapping (
    id VARCHAR PRIMARY KEY,
    real_account_id VARCHAR NOT NULL,
    controlled_account_id VARCHAR NOT NULL,
    real_account_name VARCHAR NOT NULL,
    guild_id VARCHAR NOT NULL
);
