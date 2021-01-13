use futures_util::TryStreamExt;
use licoricedev::client::{Lichess};
use licoricedev::models::board::{Event, BoardState};
use licoricedev::models::board::Challengee::{LightUser, StockFish};

use shakmaty::{Chess, Position, Color};
use shakmaty::uci::{Uci, IllegalUciError};
use shakmaty::fen;
use shakmaty::fen::Fen;

use rand::prelude::*;

use uciengine::uciengine::*;

/// make uci moves from starting position and return fen of resulting position
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
	/// lichess client
	pub lichess: Lichess,	
	/// bot name
	pub bot_name: String,
	/// engine name
	pub engine_name: String,
}

/// lichess bot implementation
impl LichessBot {
	/// create new lichess bot
	pub fn new() -> LichessBot {
		LichessBot {
			lichess: Lichess::new(std::env::var("RUST_BOT_TOKEN").unwrap()),
			bot_name: std::env::var("RUST_BOT_NAME").unwrap(),
			engine_name: std::env::var("RUST_BOT_ENGINE_NAME").unwrap(),
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

		let engine = UciEngine::new(self.engine_name.to_owned());
		
		while let Some(game_event) = game_stream.try_next().await? {
			println!("{:?}", game_event);

			let white:String;
			let black:String;			
			
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
					
					if self.bot_name == black {
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

			if state_opt.is_some() {
				let state = state_opt.unwrap();

				println!("state {:?}", state);

				let fen = make_uci_moves(state.moves.as_str())?;

				println!("fen {}", fen);

				let setup: Fen = fen.parse()?;
				let pos: Chess = setup.position(shakmaty::CastlingMode::Standard)?;

				let legals = pos.legals();

				let rand_move = legals.choose(&mut rand::thread_rng()).unwrap();

				let rand_uci = Uci::from_standard(&rand_move).to_string();

				println!("rand uci {}", rand_uci);

				let turn = setup.turn;

				println!("turn {:?}", turn);

				let bot_turn = ( ( turn == Color::White ) && bot_white ) || ( ( turn == Color::Black ) && !bot_white );

				println!("bot turn {}", bot_turn);

				if bot_turn {
					let id = game_id.to_owned();

					let moves = format!("{}", state.moves);

					let go_job = GoJob::new()
						.uci_opt("UCI_Variant", "chess")
						.pos_startpos()
						.pos_moves(moves)
						.tc(Timecontrol{
							wtime: state.wtime as usize,
							winc: state.winc as usize,
							btime: state.btime as usize,
							binc: state.binc as usize
						})
					;

					println!("engine start thinking on {:?}", go_job);

					let go_result = engine.go(go_job).recv().await;

					println!("thinking result {:?}", go_result);

					let mut bestmove = rand_uci;

					if let Some(go_result) = go_result {
						if let Some(bm) = go_result.bestmove {
							bestmove = bm;
						}
					}

					println!("making move {}", bestmove);

					let result = self.lichess.make_a_bot_move(id.as_str(), bestmove.as_str(), false).await;

					println!("make move result {:?}", result);
				}
			}
		}

		engine.quit();

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
