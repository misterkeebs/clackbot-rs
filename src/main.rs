mod http_server;
mod twitch;
mod wpm;

use std::sync::{atomic::AtomicU8, Arc};

use tokio::{
    sync::{OnceCell, RwLock},
    task::JoinHandle,
};
use twitch::{init_events, init_twitch};
use wpm::WpmGame;

pub static WPM_GAME: OnceCell<Arc<RwLock<WpmGame>>> = OnceCell::const_new();
pub static LIVE_WPM: OnceCell<Arc<AtomicU8>> = OnceCell::const_new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    println!("Init WPM game...");
    init_wpm_game().await;
    println!("Init HTTP server...");
    let http_join_handle = init_http_server().await;
    init_events().await?;
    println!("Init Twitch...");
    init_twitch().await;

    http_join_handle.await??;

    Ok(())
}

async fn init_wpm_game() {
    let wpm_game = Arc::new(RwLock::new(WpmGame::new()));
    WPM_GAME.set(wpm_game).unwrap();

    let live_wpm = Arc::new(AtomicU8::new(0));
    LIVE_WPM.set(live_wpm).unwrap();
}

async fn init_http_server() -> JoinHandle<Result<(), anyhow::Error>> {
    tokio::spawn(async move {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        http_server::start(port).await
    })
}
