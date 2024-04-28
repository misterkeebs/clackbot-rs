use std::env;

use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use twitch_api::{
    eventsub::{channel::ChannelFollowV2, Message, Transport},
    helix::users::GetUsersRequest,
    pubsub::{PubSubClient, PubSubMessage, PubSubRequest, PubSubTopic},
    twitch_oauth2::{AppAccessToken, Scope},
    types, TwitchClient,
};

pub async fn init_events() -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let client_id = env::var("TWITCH_CLIENT_ID").unwrap();
    let client_secret = env::var("TWITCH_SECRET").unwrap();
    let channel_id = env::var("TWITCH_CHANNEL_ID").unwrap();

    let scopes = vec![Scope::ChannelReadSubscriptions];
    let token = AppAccessToken::get_app_access_token(
        &client,
        client_id.into(),
        client_secret.into(),
        scopes,
    )
    .await?;

    let user_id = types::UserId::new(channel_id);
    let topic = PubSubTopic::VideoPlayback(user_id);
    let request = PubSubRequest::Listen(topic);

    let (mut socket, _) = connect_async("wss://pubsub-edge.twitch.tv").await?;
    socket
        .send(Message::Text(serde_json::to_string(&request)?))
        .await?;

    let mut pubsub_client = PubSubClient::new();

    loop {
        if let Some(message) = pubsub_client.handle_message(&mut socket).await? {
            match message {
                PubSubMessage::VideoPlayback(playback) => {
                    println!("Received video playback event: {:?}", playback);
                }
                _ => {}
            }
        }
    }
}

pub async fn init_events2() -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    // let token = AccessToken::new(env::var("TWITCH_TOKEN").unwrap());
    // let token = UserToken::from_token(&client, token).await?;
    let secret = env::var("TWITCH_SECRET").unwrap();
    let client_id = env::var("TWITCH_CLIENT_ID").unwrap();
    let channel = env::var("TWITCH_CHANNEL").unwrap();
    // let scope = Scope::parse("channel:read:subscriptions");
    let scope = Scope::all();
    let token =
        AppAccessToken::get_app_access_token(&client, client_id.into(), secret.into(), scope)
            .await?;

    let client: TwitchClient<reqwest::Client> = TwitchClient::default();
    let logins: &[&types::UserNameRef] = &[(&channel).into()];
    let request = GetUsersRequest::logins(logins);

    let response = client.helix.req_get(request, &token).await?;

    let user_ids = response
        .data
        .into_iter()
        .map(|user| user.id)
        .collect::<Vec<_>>();
    println!("user ids: {:?}", user_ids);

    let user_id = &user_ids[0];
    let event = ChannelFollowV2::new(user_id.clone(), user_id.clone());
    let transport = Transport::webhook(
        "https://example.org/eventsub/channelfollow",
        String::from(env::var("TWITCH_SECRET").unwrap()),
    );

    let event_information = client
        .helix
        .create_eventsub_subscription(event, transport, &token)
        .await?;

    println!("event id: {:?}", event_information.id);
    Ok(())
}
