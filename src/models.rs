use chrono::naive::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use regex::Regex;

use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::twitch::{Client, Redemption};

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
    pub clacks: i32,
    pub modified_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

impl User {
    pub async fn process(
        &self,
        redemption: Redemption,
        conn: &mut AsyncPgConnection,
        client: &Client,
    ) -> anyhow::Result<()> {
        let Some(amount) = extract_int(&redemption.reward.title) else {
            println!(
                "Failed to extract amount from reward title: {:#?}",
                redemption
            );
            return Ok(());
        };

        diesel::update(users)
            .filter(users::id.eq(&self.id))
            .set(users::clacks.eq(clacks + amount))
            .get_result::<User>(conn)
            .await?;
        redemption.complete(client).await?;

        Ok(())
    }
}

fn extract_int(input: &str) -> Option<i32> {
    let re = Regex::new(r"\d+").unwrap();
    match re.find(input) {
        Some(matched) => matched.as_str().parse::<i32>().ok(),
        None => None,
    }
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
