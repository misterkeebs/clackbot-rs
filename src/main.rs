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
use twitch::init_twitch;
use wpm::WpmGame;

use crate::{db::connect, discord::init_discord, twitch::init_events};

pub static WPM_GAME: OnceCell<Arc<RwLock<WpmGame>>> = OnceCell::const_new();
pub static LIVE_WPM: OnceCell<Arc<AtomicU8>> = OnceCell::const_new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "clackbot=debug"); // Use MY_APP_LOG or default to debug
    env_logger::init_from_env(env);

    log::debug!("Connecting to database...");
    let pool = connect().await?;

    log::debug!("Init WPM game...");
    init_wpm_game().await;

    log::debug!("Init HTTP server...");
    let http_join_handle = init_http_server(&pool).await;

    log::debug!("Init twitch event handler...");
    let events_join_handle = init_events(&pool).await;

    log::debug!("Init Discord...");
    init_discord().await?;

    log::debug!("Init Twitch...");
    init_twitch().await;

    let (_http_result, _events_result) = tokio::try_join!(http_join_handle, events_join_handle)?;

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
