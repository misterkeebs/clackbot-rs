mod error;

use std::{collections::HashMap, sync::atomic::Ordering};

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use tower_http::services::ServeDir;

use crate::{twitch::TWITCH, LIVE_WPM, WPM_GAME};

#[derive(Clone)]
struct State {
    pool: PgPool,
}

pub async fn start(port: String, pool: PgPool) -> anyhow::Result<()> {
    let app = Router::new()
        .nest_service("/", ServeDir::new("public"))
        .route("/standings", get(get_standings))
        .route("/liveWpm", get(live_wpm))
        .route("/finishWpm", get(register_wpm))
        .route("/setLiveWpm", get(set_live_wpm))
        .route("/discord/callback", get(discord_callback))
        .layer(Extension(State { pool }));

    let address = format!("0.0.0.0:{}", port);
    println!("Listening on: {}", address);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn discord_callback(
    q: Query<serde_json::Value>,
    Extension(state): Extension<State>,
) -> Result<String, error::Error> {
    let code = q.get("code").and_then(|v| v.as_str());

    match code {
        Some(code) => {
            process_discord_callback(&state.pool, code).await?;
            Ok("Successfully linked your Twitch account to Discord".to_string())
        }
        None => Err(error::Error::InvalidRequest)?,
    }
}

async fn set_live_wpm(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    if let Some(wpm) = params.get("wpm").and_then(|w| w.parse::<u8>().ok()) {
        let wpm_game = WPM_GAME.get().unwrap();
        let wpm_game = wpm_game.read().await;
        if !wpm_game.is_running() {
            return StatusCode::OK;
        }

        println!("Setting live WPM to: {}", wpm);

        LIVE_WPM.get().unwrap().store(wpm, Ordering::Relaxed);

        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}

async fn get_standings() -> Json<serde_json::Value> {
    let wpm_game = WPM_GAME.get().unwrap();
    let wpm_game = wpm_game.read().await;

    let wpm = LIVE_WPM.get().unwrap().load(Ordering::Relaxed);
    println!("current WPM: {}", wpm);

    let players = wpm_game
        .guesses()
        .iter()
        .map(|g| (g.user.clone(), g.wpm, (wpm as i32 - g.wpm as i32).abs()))
        .collect::<Vec<_>>();

    // sort by distance to live WPM
    let mut players = players;
    players.sort_by(|a, b| a.2.cmp(&b.2));

    let res = json!({
        "running": wpm_game.is_running(),
        "players": players,
        "liveWpm": wpm,
        "lastWinner": wpm_game.last_winner(),
    });

    Json(res)
}

async fn live_wpm() -> Json<u8> {
    let wpm = LIVE_WPM.get().unwrap().load(Ordering::Relaxed);
    Json(wpm)
}

async fn register_wpm(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let Some(wpm) = params.get("wpm").and_then(|w| w.parse::<usize>().ok()) else {
        return StatusCode::BAD_REQUEST;
    };

    let wpm_game = WPM_GAME.get().unwrap();
    let mut wpm_game = wpm_game.write().await;

    if !wpm_game.is_running() {
        return StatusCode::OK;
    }

    let winner = wpm_game.winner(wpm);

    let twitch = TWITCH.get().unwrap();

    match winner {
        Some((winner, winner_wpm)) => {
            twitch
                .send(format!(
                    "typing test ended with {} WPM. The the winner is {} with a guess of {} WPM.",
                    wpm, winner, winner_wpm
                ))
                .await;
        }
        None => {
            twitch
                .send(format!(
                    "MrKeebs did {wpm} WPM but still is forever alone: no guesses :-("
                ))
                .await
        }
    }

    StatusCode::OK
}

async fn process_discord_callback(pool: &PgPool, code: &str) -> Result<(), error::Error> {
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
    println!("Response: {:?}", body);

    let Some(access_token) = body["access_token"].as_str() else {
        return Err(error::Error::InvalidResponse(body));
    };

    let user_res = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    let user = user_res.json::<serde_json::Value>().await?;

    println!("User: {:#?}", user);

    let conns_res = client
        .get("https://discord.com/api/users/@me/connections")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    let conns = conns_res.json::<serde_json::Value>().await?;

    println!("User: {:#?}", user);
    println!("Connections: {:#?}", conns);

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

    println!("Twitch ID: {}", twitch_id);
    println!("Twitch Name: {}", twitch_name);

    let username = user["username"].as_str().unwrap();
    let discord_id = user["id"].as_str().unwrap();
    let discord_name = user["global_name"].as_str().unwrap();

    sqlx::query!(
        "INSERT INTO users (username, discord_id, discord_name, twitch_id, twitch_name) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (discord_id) DO UPDATE SET twitch_id = $4, twitch_name = $5",
        username,
        discord_id,
        discord_name,
        twitch_id,
        twitch_name
    ).execute(pool).await?;

    Ok(())
}
