// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (id) {
        id -> Int4,
        user_id -> Int4,
        description -> Text,
        clacks -> Int4,
        modified_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        discord_id -> Nullable<Varchar>,
        #[max_length = 255]
        discord_name -> Nullable<Varchar>,
        #[max_length = 255]
        twitch_id -> Nullable<Varchar>,
        #[max_length = 255]
        twitch_name -> Nullable<Varchar>,
        clacks -> Int4,
        modified_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(transactions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    transactions,
    users,
);
