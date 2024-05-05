mod db;
mod discord;
mod http_server;
mod models;
mod schema;
mod twitch;
mod wpm;

use std::sync::{atomic::AtomicU8, Arc};

use db::Pool;
use env_logger::Env;
use tokio::{
    sync::{OnceCell, RwLock},
    task::JoinHandle,
};
use wpm::WpmGame;

use crate::{
    db::connect,
    discord::init_discord,
    twitch::{init_event_handler, init_rewards_monitor},
};

pub static WPM_GAME: OnceCell<Arc<RwLock<WpmGame>>> = OnceCell::const_new();
pub static LIVE_WPM: OnceCell<Arc<AtomicU8>> = OnceCell::const_new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let env = Env::default().filter_or("LOG_LEVEL", "clackbot=debug");
    env_logger::init_from_env(env);

    log::debug!("Connecting to database...");
    let pool = connect().await?;

    log::debug!("Init WPM game...");
    init_wpm_game().await;

    log::debug!("Init Twitch chat monitor...");
    let twitch_join_handle = init_twitch().await;

    log::debug!("Init Twitch rewards monitor...");
    let rewards_join_handle = init_rewards_monitor(&pool).await;

    log::debug!("Init Twitch event handler...");
    let (ev_prod_handle, ev_cons_handle) = init_event_handler(&pool).await?;

    log::debug!("Init HTTP server...");
    let http_join_handle = init_http_server(&pool).await;

    log::debug!("Init Discord...");
    init_discord().await?;

    // let (_twitch_result, _http_result, _rewards_result, _ev_prod_result, _ev_cons_result) = tokio::try_join!(
    _ = tokio::try_join!(
        twitch_join_handle,
        http_join_handle,
        rewards_join_handle,
        ev_prod_handle,
        ev_cons_handle,
    )?;

    Ok(())
}

async fn init_wpm_game() {
    let wpm_game = Arc::new(RwLock::new(WpmGame::new()));
    WPM_GAME.set(wpm_game).unwrap();

    let live_wpm = Arc::new(AtomicU8::new(0));
    LIVE_WPM.set(live_wpm).unwrap();
}

async fn init_http_server(pool: &Pool) -> JoinHandle<Result<(), anyhow::Error>> {
    let pool = pool.clone();
    tokio::spawn(async move {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        http_server::start(port, pool).await
    })
}

async fn init_twitch() -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        twitch::init_twitch().await?;
        Ok(())
    })
}
