#[derive(Debug)]
struct Guess {
    user: String,
    wpm: usize,
}

#[derive(Debug)]
pub struct WpmGame {
    running: bool,
    guesses: Vec<Guess>,
}

impl WpmGame {
    pub fn new() -> WpmGame {
        WpmGame {
            running: false,
            guesses: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        self.running = true;
        self.guesses.clear();
    }

    pub fn add_guess(&mut self, user: &str, wpm: usize) -> anyhow::Result<()> {
        if !self.running {
            return Err(anyhow::anyhow!(
                "there is no typing test going on at the moment"
            ));
        }

        self.guesses.push(Guess {
            user: user.to_string(),
            wpm,
        });

        Ok(())
    }

    pub fn winner(&mut self, wpm: usize) -> Option<(String, usize)> {
        if self.guesses.len() < 1 {
            self.running = false;
            return None;
        }

        let mut distances = self
            .guesses
            .iter()
            .map(|g| (g.user.clone(), (wpm as isize - g.wpm as isize).abs()))
            .collect::<Vec<_>>();
        let mut winner = distances.remove(0);
        for d in distances {
            if d.1 < winner.1 {
                winner = d;
            }
        }
        let wpm = self
            .guesses
            .iter()
            .find(|g| g.user == winner.0)
            .unwrap()
            .wpm;

        Some((winner.0, wpm))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_running() {
        let mut game = WpmGame::new();
        assert!(game.add_guess("misterkeebs", 100).is_err());
    }

    #[test]
    fn test_tied_winner_under() {
        let mut game = WpmGame::new();
        game.start();
        game.add_guess("misterkeebs", 98).unwrap();
        game.add_guess("forbidden404", 103).unwrap();
        game.add_guess("owesome", 99).unwrap();
        game.add_guess("purryoverlord", 99).unwrap();

        assert_eq!(game.winner(100), Some(("owesome".to_string(), 99)));
    }

    #[test]
    fn test_winner_over() {
        let mut game = WpmGame::new();
        game.start();
        game.add_guess("forbidden404", 103).unwrap();
        game.add_guess("purryoverlord", 99).unwrap();
        game.add_guess("misterkeebs", 97).unwrap();

        // purry wins because she guessed first
        assert_eq!(game.winner(98), Some(("purryoverlord".to_string(), 99)));
    }

    #[test]
    fn test_winner_exact() {
        let mut game = WpmGame::new();
        game.start();
        game.add_guess("misterkeebs", 98).unwrap();
        game.add_guess("forbidden404", 103).unwrap();
        game.add_guess("owesome", 99).unwrap();
        game.add_guess("purryoverlord", 99).unwrap();

        assert_eq!(game.winner(99), Some(("owesome".to_string(), 99)));
    }

    #[test]
    fn test_no_guesses() {
        let mut game = WpmGame::new();
        game.start();
        assert!(game.winner(99).is_none());
    }
}
