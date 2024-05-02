use diesel_async::RunQueryDsl;

use crate::{db::Pool, models::NewUser, schema::users};

use super::error;

pub async fn process_discord_callback(pool: &Pool, code: &str) -> Result<String, error::Error> {
    let mut conn = pool.get().await?;

    let client_id = std::env::var("DISCORD_CLIENT_ID").expect("missing DISCORD_CLIENT_ID");
    let client_secret = std::env::var("DISCORD_SECRET").expect("missing DISCORD_SECRET");
    let redirect_uri = std::env::var("DISCORD_REDIRECT_URI").expect("missing DISCORD_REDIRECT_URI");

    let client = reqwest::Client::new();
    let res = client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "authorization_code".to_string()),
            ("code", code.to_string()),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await?;

    let body = res.json::<serde_json::Value>().await?;
    log::trace!("Response: {:?}", body);

    let Some(access_token) = body["access_token"].as_str() else {
        return Err(error::Error::InvalidResponse(body));
    };

    let user_res = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    let user = user_res.json::<serde_json::Value>().await?;

    log::trace!("User: {:#?}", user);

    let conns_res = client
        .get("https://discord.com/api/users/@me/connections")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    let conns = conns_res.json::<serde_json::Value>().await?;

    log::trace!("User: {:#?}", user);
    log::trace!("Connections: {:#?}", conns);

    let twitch = conns
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["type"] == "twitch");

    let Some(twitch) = twitch else {
        return Err(error::Error::TwitterNotLinked);
    };

    let twitch_id = twitch["id"].as_str().unwrap();
    let twitch_name = twitch["name"].as_str().unwrap();
    let username = user["username"].as_str().unwrap();
    let discord_id = Some(user["id"].as_str().unwrap());
    let discord_name = Some(user["global_name"].as_str().unwrap());

    let user = NewUser {
        username,
        discord_id,
        discord_name,
        twitch_id: Some(twitch_id),
        twitch_name: Some(twitch_name),
    };
    diesel::insert_into(users::table)
        .values(&user)
        .on_conflict(users::discord_id)
        .do_update()
        .set(&user)
        .execute(&mut conn)
        .await?;

    Ok(twitch_name.to_string())
}
