mod http_server;
mod twitch;
mod wpm;

use std::sync::Arc;

use tokio::{
    sync::{Mutex, OnceCell},
    task::{JoinError, JoinHandle},
};
use twitch::init_twitch;
use wpm::WpmGame;

pub static WPM_GAME: OnceCell<Arc<Mutex<WpmGame>>> = OnceCell::const_new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    println!("Init WPM game...");
    init_wpm_game().await;
    println!("Init HTTP server...");
    let http_join_handle = init_http_server().await;
    println!("Init Twitch...");
    init_twitch().await;

    http_join_handle.await??;

    Ok(())
}

async fn init_wpm_game() {
    let wpm_game = Arc::new(Mutex::new(WpmGame::new()));
    WPM_GAME.set(wpm_game).unwrap();
}

async fn init_http_server() -> JoinHandle<Result<(), anyhow::Error>> {
    tokio::spawn(async move {
        let addr = std::env::var("HTTP_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        http_server::start(addr).await
    })
}
