// @generated automatically by Diesel CLI.

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
        modified_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
    }
}
