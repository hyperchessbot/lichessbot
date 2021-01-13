use futures_util::TryStreamExt;
use licoricedev::client::{Lichess};
use licoricedev::models::board::{Event, BoardState};
use licoricedev::models::board::Challengee::{LightUser, StockFish};

use shakmaty::{Chess, Position};
use shakmaty::uci::{Uci, IllegalUciError};
use shakmaty::fen;

pub fn make_uci_moves(ucis_str: &str) -> Result<String, Box<dyn std::error::Error>> {
	let mut pos = Chess::default();
	if ucis_str.len() > 0 {
		for uci_str in ucis_str.split(" ") {
			let uci: Uci = uci_str.parse()?;						
			let m = uci.to_move(&pos.to_owned())?;		
			match pos.to_owned().play(&m) {
				Ok(newpos) => pos = newpos,
				Err(_) => return Err(Box::new(IllegalUciError)),
			}		
		}
	}
	Ok(fen::fen(&pos))
}

/// lichess bot
pub struct LichessBot {
	lichess: Lichess
}

/// lichess bot implementation
impl LichessBot {
	/// create new lichess bot
	pub fn new() -> LichessBot {
		LichessBot {
			lichess: Lichess::new(std::env::var("RUST_BOT_TOKEN").unwrap())
		}
	}

	/// play game
	async fn play_game(&mut self, game_id: String) -> Result<(), Box::<dyn std::error::Error>> {
		println!("playing game {}", game_id);

		let mut game_stream = self.lichess
			.stream_bot_game_state(&game_id)
			.await
			.unwrap();
		
		let mut bot_white = true;					
		
		while let Some(game_event) = game_stream.try_next().await? {
			println!("{:?}", game_event);

			let white:String;
			let black:String;
			let bot = std::env::var("RUST_BOT_NAME").unwrap();				
			
			let state_opt = match game_event {
				BoardState::GameFull ( game_full ) => {
					println!("game full {:?}", game_full);
					
					white = match game_full.white {
						LightUser(user) => user.username,
						StockFish(sf) => format!("Stockfish AI level {}", sf.ai_level)
					};

					black = match game_full.black {
						LightUser(user) => user.username,
						StockFish(sf) => format!("Stockfish AI level {}", sf.ai_level)
					};
					
					if bot == black {
						bot_white = false;
					}
					
					println!("**************\n{} - {} bot white {}\n**************", white, black, bot_white);
					
					Some(game_full.state)
				},
				BoardState::GameState ( game_state ) => {
					Some(game_state)
				},
				_ => {
					println!("undhandled game event {:?}", game_event);

					None
				}
			};

			if state_opt.is_none() {
				return Ok(())
			}

			let state = state_opt.unwrap();

			let fen = make_uci_moves(state.moves.as_str())?;

			println!("fen {}", fen);

			println!("state {:?}", state);
		}

		Ok(())
	}

	/// process event stream event
	async fn process_event_stream_event(&mut self, event: Event) -> Result<(), Box::<dyn std::error::Error>> {
		println!("event {:?}", event);

		match event {
			Event::Challenge { challenge } => {
				println!("incoming challenge {:?}", challenge.id);
				
				if challenge.variant.key == "standard" {
					if challenge.speed == "correspondence" {
						println!("rejecting challenge, correspondence");
					} else {
						println!("accepting challenge, response {:?}",
							self.lichess.challenge_accept(&challenge.id).await);															}
				} else {
					println!("rejecting challenge, wrong variant {}", challenge.variant.key);
				}
			},
			Event::GameStart { game } => {
				let game_id = format!("{}", game.id);

				println!("game started {}", game_id);

				println!("play game result {:?}", self.play_game(game_id).await);
			}
			_ => {
				println!("unhandled event")
			}
		};

		Ok(())
	}

	/// stream events
	pub async fn stream(&mut self) -> Result<(), Box::<dyn std::error::Error>> {
		let mut event_stream = self.lichess
			.stream_incoming_events()
			.await
			.unwrap();

		while let Some(event) = event_stream.try_next().await? {			
	    	self.process_event_stream_event(event).await?;
	    }

	    Ok(())
	}
}
