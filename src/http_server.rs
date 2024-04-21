use std::sync::atomic::Ordering;

use serde_json::json;

use crate::{twitch::TWITCH, LIVE_WPM, WPM_GAME};

pub async fn start(port: String) -> anyhow::Result<()> {
    let mut app = tide::new();
    app.at("/").serve_dir("public/")?;
    app.at("/standings").get(get_standings);
    app.at("/liveWpm").get(|_| async move {
        let wpm = LIVE_WPM.get().unwrap().load(Ordering::Relaxed).to_string();
        Ok(wpm)
    });
    app.at("/finishWpm").get(register_wpm);
    app.at("/setLiveWpm").get(set_live_wpm);
    let address = format!("0.0.0.0:{port}");
    println!("Listening on: {}", address);
    app.listen(address).await?;

    Ok(())
}

async fn set_live_wpm(req: tide::Request<()>) -> tide::Result {
    println!(
        "set_live_wpm: {:?}",
        req.url()
            .query_pairs()
            .map(|(k, v)| (k, v))
            .collect::<Vec<_>>()
    );
    let Some(query) = req.url().query_pairs().next() else {
        println!("no query");
        return Ok(tide::Response::new(400));
    };

    let wpm_game = WPM_GAME.get().unwrap();
    let wpm_game = wpm_game.read().await;
    if !wpm_game.is_running() {
        return Ok(tide::Response::new(200));
    }

    let wpm = query.1;
    let wpm = wpm.parse::<u8>()?;
    println!("Setting live WPM to: {}", wpm);

    LIVE_WPM.get().unwrap().store(wpm, Ordering::Relaxed);

    Ok(tide::Response::new(200))
}

async fn get_standings(_: tide::Request<()>) -> tide::Result {
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

    Ok(res.into())
}

async fn register_wpm(req: tide::Request<()>) -> tide::Result {
    println!("register_wpm");

    let Some(query) = req.url().query_pairs().next() else {
        return Ok(tide::Response::new(400));
    };

    let wpm = query.1;
    let wpm = wpm.parse::<usize>()?;

    let wpm_game = WPM_GAME.get().unwrap();
    let mut wpm_game = wpm_game.write().await;

    if !wpm_game.is_running() {
        return Ok(tide::Response::new(200));
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

    Ok(tide::Response::new(200))
}
