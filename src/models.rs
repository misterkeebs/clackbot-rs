use chrono::naive::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::users;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub discord_id: Option<String>,
    pub discord_name: Option<String>,
    pub twitch_id: Option<String>,
    pub twitch_name: Option<String>,
    pub modified_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub discord_id: Option<&'a str>,
    pub discord_name: Option<&'a str>,
    pub twitch_id: Option<&'a str>,
    pub twitch_name: Option<&'a str>,
}
