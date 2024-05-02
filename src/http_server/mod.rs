mod discord;
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
use tower_http::services::ServeDir;

use crate::{db::Pool, twitch::TWITCH, LIVE_WPM, WPM_GAME};

use self::discord::process_discord_callback;

#[derive(Clone)]
struct State {
    pool: Pool,
}

pub async fn start(port: String, pool: Pool) -> anyhow::Result<()> {
    let app = Router::new()
        .nest_service("/", ServeDir::new("public"))
        .route("/standings", get(get_standings))
        .route("/liveWpm", get(live_wpm))
        .route("/finishWpm", get(register_wpm))
        .route("/setLiveWpm", get(set_live_wpm))
        .route("/discord/callback", get(discord_callback))
        .layer(Extension(State { pool }));

    let address = format!("0.0.0.0:{}", port);
    log::info!("Listening on: {}", address);
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
            let username = process_discord_callback(&state.pool, code).await?;
            Ok(format!(
                "Successfully linked your Twitch account {username}. You can now close this window."
            ))
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

        log::trace!("Setting live WPM to: {}", wpm);

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
    log::debug!("current WPM: {}", wpm);

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
