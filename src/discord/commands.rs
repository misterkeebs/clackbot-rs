use poise::serenity_prelude::*;

struct Data {} // User data, which is stored and accessible in all command invocations
type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn init_discord() -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![link()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
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
        "Hello, {}! Please [click here to link your Twitch account]({}).",
        u.name, oauth_url
    );
    ctx.say(response).await?;

    Ok(())
}
