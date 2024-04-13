use crate::{twitch::TWITCH, WPM_GAME};

pub async fn start(port: String) -> anyhow::Result<()> {
    let mut app = tide::new();
    app.at("/").get(|_| async { Ok("So empty...") });
    app.at("/wpm").get(register_wpm);
    let address = format!("0.0.0.0:{port}");
    println!("Listening on: {}", address);
    app.listen(address).await?;

    Ok(())
}

async fn register_wpm(req: tide::Request<()>) -> tide::Result {
    println!("register_wpm");
    let Some(query) = req.url().query_pairs().next() else {
        return Ok(tide::Response::new(400));
    };

    let wpm = query.1;
    let wpm = wpm.parse::<usize>()?;

    let wpm_game = WPM_GAME.get().unwrap();
    let mut wpm_game = wpm_game.lock().await;
    let winner = wpm_game.winner(wpm);

    let twitch = TWITCH.get().unwrap();

    match winner {
        Some((winner, winner_wpm)) => {
            twitch
                .send(format!(
                    "typing test ended with {} WPM. The the winner is {} with a WPM of {}.",
                    wpm, winner, winner_wpm
                ))
                .await;
        }
        None => {
            twitch
                .send("Forever alone: no guesses :-(".to_string())
                .await
        }
    }

    Ok(tide::Response::new(200))
}
