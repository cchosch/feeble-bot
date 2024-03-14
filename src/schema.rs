// @generated automatically by Diesel CLI.

diesel::table! {
    account_mapping (id) {
        id -> Varchar,
        mapped_discord_id -> Varchar,
        controlled_discord_id -> Varchar,
        controlled_internal_id -> Varchar,
        controlled_username -> Varchar,
    }
}

diesel::table! {
    controlled_account (id) {
        id -> Varchar,
        discord_id -> Varchar,
        username -> Varchar,
        token -> Varchar,
        created_by -> Varchar,
    }
}

diesel::table! {
    discord_account (id) {
        id -> Varchar,
        account_id -> Varchar,
        account_name -> Varchar,
        account_token -> Varchar,
        created_by -> Varchar,
    }
}

diesel::table! {
    sessions (sess_id) {
        sess_id -> Text,
        sess_cookie -> Text,
        expiry -> Timestamptz,
        uid -> Nullable<Text>,
        data -> Nullable<Json>,
    }
}

diesel::table! {
    users (id) {
        id -> Varchar,
        username -> Text,
        password -> Text,
        email -> Text,
        created_at -> Timestamptz,
        verified_email -> Bool,
        banned -> Bool,
        kicked_until -> Timestamptz,
        flags -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    account_mapping,
    controlled_account,
    discord_account,
    sessions,
    users,
);
