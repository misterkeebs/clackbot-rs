use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use tokio::task::JoinHandle;

use crate::{
    db::Pool,
    models::User,
    twitch::{client::Client, types::Reward},
};

const REWARDS: &[(&str, u16, &str)] = &[
    ("3 Clacks", 100, "3 Clacks"),
    ("10 Clacks", 350, "5 Clacks"),
    ("20 Clacks", 600, "20 Clacks"),
];

struct EventHandler {
    client: Client,
    rewards: Vec<Reward>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            rewards: Vec::new(),
        }
    }

    pub async fn monitor_reward_redemptions(&mut self, pool: Pool) -> anyhow::Result<()> {
        self.init_rewards().await?;
        self.manage_rewards().await?;

        loop {
            log::trace!("Processing redemptions...");
            let mut conn = pool.get().await?;
            self.process_redemptions(&mut conn).await?;
            // waits for 5 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    async fn init_rewards(&mut self) -> anyhow::Result<()> {
        let rewards = self.client.get_rewards().await?;
        self.rewards = rewards.data;

        Ok(())
    }

    async fn manage_rewards(&self) -> anyhow::Result<()> {
        for (title, cost, prompt) in REWARDS {
            if self.rewards.iter().any(|r| r.title == *title) {
                continue;
            }
            log::trace!("Reward title = {}", title);
            let reward = Reward::builder(title.to_string(), *cost as i32)
                .prompt(prompt.to_string())
                .build();
            reward.create(&self.client).await?;
        }

        Ok(())
    }

    async fn process_redemptions(&self, conn: &mut AsyncPgConnection) -> anyhow::Result<()> {
        use crate::schema::users::dsl::*;

        for reward in &self.rewards {
            let redemptions = reward.get_pending_redemptions(&self.client).await?;
            for redemption in redemptions.data {
                log::trace!("found redemption = {:#?}", redemption);
                let res = users
                    .filter(twitch_id.eq(&redemption.user_id))
                    .select(User::as_select())
                    .load(conn)
                    .await?;
                log::trace!("user = {:#?}", res);
                if res.len() != 1 {
                    continue;
                }
                let user = &res[0];
                user.process_redemption(redemption, conn, &self.client)
                    .await?;
            }
        }

        Ok(())
    }
}

pub async fn init_rewards_monitor(pool: &Pool) -> JoinHandle<anyhow::Result<()>> {
    let pool = pool.clone();

    tokio::spawn(async move {
        let mut handler = EventHandler::new();
        handler.monitor_reward_redemptions(pool).await
    })
}
