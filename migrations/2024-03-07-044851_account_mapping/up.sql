-- Your SQL goes here
CREATE TABLE discord_account (
    id VARCHAR PRIMARY KEY,
    account_id VARCHAR NOT NULL,
    account_name VARCHAR NOT NULL,
    account_token VARCHAR NOT NULL,
    created_by VARCHAR NOT NULL
);

CREATE TABLE account_mapping (
    id VARCHAR PRIMARY KEY,
    real_account_id VARCHAR NOT NULL,
    controlled_account_id VARCHAR NOT NULL
);
