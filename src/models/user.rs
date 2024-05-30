use chrono::naive::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use regex::Regex;

use crate::schema::users::dsl::*;
use crate::schema::{transactions, users};
use crate::twitch::{Client, Redemption, TWITCH};

use super::NewTransaction;

#[derive(Queryable, Selectable, Debug, Clone)]
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
    pub async fn get_by_twitch_id(
        conn: &mut AsyncPgConnection,
        tid: String,
    ) -> anyhow::Result<Option<User>> {
        let data: Vec<User> = users::table
            .filter(users::twitch_id.eq(tid))
            .select(User::as_select())
            .load(conn)
            .await?;

        if data.len() < 1 {
            return Ok(None);
        }

        Ok(Some(data[0].clone()))
    }

    pub async fn get_by_discord_id(
        conn: &mut AsyncPgConnection,
        did: String,
    ) -> anyhow::Result<Option<User>> {
        let data: Vec<User> = users::table
            .filter(users::discord_id.eq(did))
            .select(User::as_select())
            .load(conn)
            .await?;

        if data.len() < 1 {
            return Ok(None);
        }

        Ok(Some(data[0].clone()))
    }

    pub async fn get_or_create_discord_user(
        conn: &mut AsyncPgConnection,
        discord_user: &poise::serenity_prelude::model::user::User,
    ) -> anyhow::Result<User> {
        let did = discord_user.id.get().to_string();
        let user = User::get_by_discord_id(conn, did.clone()).await?;

        if let Some(user) = user {
            return Ok(user);
        }

        let new_user = NewUser {
            username: &discord_user.name,
            discord_id: Some(&did),
            discord_name: discord_user.global_name.as_deref(),
            twitch_id: None,
            twitch_name: None,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)
            .await?;

        let Some(user) = User::get_by_discord_id(conn, did).await? else {
            return Err(anyhow::anyhow!("Failed to create user"));
        };

        Ok(user)
    }

    pub async fn process_redemption(
        &self,
        redemption: Redemption,
        conn: &mut AsyncPgConnection,
        client: &Client,
    ) -> anyhow::Result<()> {
        let Some(amount) = extract_int(&redemption.reward.title) else {
            log::warn!(
                "Failed to extract amount from reward title: {:#?}",
                redemption
            );
            return Ok(());
        };

        let transaction = NewTransaction {
            user_id: self.id,
            description: format!("Redeemed Twitch reward '{}'", redemption.reward.title),
            clacks: amount,
        };

        diesel::insert_into(transactions::table)
            .values(&transaction)
            .execute(conn)
            .await?;

        diesel::update(users)
            .filter(users::id.eq(&self.id))
            .set(users::clacks.eq(clacks + amount))
            .get_result::<User>(conn)
            .await?;
        redemption.complete(client).await?;

        TWITCH
            .get()
            .unwrap()
            .send(
                format!(
                    "{} your \"{}\" reward has been processed! You've got credited {} clacks, you now have {} clacks.",
                    self.twitch_name.as_ref().unwrap_or(&self.username),
                    redemption.reward.title,
                    amount,
                    self.clacks + amount
                ),
            )
            .await;

        Ok(())
    }

    pub async fn add_clacks(
        &self,
        conn: &mut AsyncPgConnection,
        description: &str,
        amount: i32,
    ) -> anyhow::Result<()> {
        let transaction = NewTransaction {
            user_id: self.id,
            description: description.to_string(),
            clacks: amount,
        };

        diesel::insert_into(transactions::table)
            .values(&transaction)
            .execute(conn)
            .await?;

        diesel::update(users)
            .filter(users::id.eq(&self.id))
            .set(users::clacks.eq(clacks + amount))
            .get_result::<User>(conn)
            .await?;

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
