use poise::serenity_prelude::*;

use crate::{db::Pool, models::User};

struct Data {
    pool: Pool,
}

impl Data {
    async fn get_user(&self, id: u64) -> anyhow::Result<Option<User>> {
        let mut conn = self.pool.get().await?;
        let id = id.to_string();
        let user = User::get_by_discord_id(&mut conn, id).await?;
        Ok(user)
    }
}

type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn init_discord(pool: &Pool) -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = GatewayIntents::non_privileged();

    let pool = pool.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![link(), clacks()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { pool })
            })
        })
        .build();

    // Clone the client to use inside the signal handler
    let client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn link(ctx: Context<'_>) -> Result<(), Error> {
    let u = ctx.author();

    // Generate the OAuth2 URL
    let client_id = std::env::var("DISCORD_CLIENT_ID").expect("missing DISCORD_CLIENT_ID");
    let redirect_uri = std::env::var("DISCORD_REDIRECT_URI").expect("missing DISCORD_REDIRECT_URI");
    let redirect_uri = urlencoding::encode(&redirect_uri);
    let oauth_url = format!("https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify%20connections", client_id, redirect_uri);

    // Send the OAuth2 URL to the user
    let response = format!(
        "In order to link your Twitch account, [please click here]({}).",
        oauth_url
    );

    // Create a DM channel and send the message
    let channel = u.create_dm_channel(&ctx).await?;
    channel.say(&ctx, &response).await?;

    ctx.reply("I've sent you a DM with instructions.").await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn clacks(ctx: Context<'_>) -> Result<(), Error> {
    match ctx.data().get_user(ctx.author().id.get()).await {
        Ok(Some(user)) => {
            ctx.reply(format!("You have {} clacks.", user.clacks))
                .await?;
        }
        Ok(None) => {
            ctx.reply("You have no clacks.").await?;
        }
        Err(e) => {
            log::error!("Error getting user: {:?}", e);
            ctx.reply(format!(
                "An error occurred trying to retrieve your user: {}",
                e
            ))
            .await?;
        }
    }

    Ok(())
}
