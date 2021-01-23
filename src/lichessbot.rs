use log::{debug, log_enabled, info, Level};

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
use pgnparse::parser::*;
use envor::envor::*;

/// make uci moves from starting position and return fen of resulting position
pub fn make_uci_moves<T>(ucis_str: T) -> Result<(String, String), Box<dyn std::error::Error>>
where T: core::fmt::Display {
	let ucis_str = format!("{}", ucis_str);

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
	Ok((fen::fen(&pos), fen::epd(&pos)))
}

/// bot state
pub struct BotState {
	/// current fen
	pub current_fen: Option<String>
}

/// lichess bot
pub struct LichessBot {
	/// lichess client
	pub lichess: Lichess,	
	/// lichess username of bot
	pub bot_name: String,
	/// engine executable name ( optional )
	pub engine_name: Option<String>,
	/// uci options
	pub uci_options: std::collections::HashMap<String, String>,
	/// enable classical
	pub enable_classical: bool,
	/// enable rapid
	pub enable_rapid: bool,
	/// disable blitz
	pub disable_blitz: bool,
	/// disable bullet
	pub disable_bullet: bool,
	/// enable ultrabullet
	pub enable_ultrabullet: bool,
	/// enable casual
	pub enable_casual: bool,
	/// disable rated
	pub disable_rated: bool,
	/// book
	pub book: Book,
	/// state
	pub state: std::sync::Arc<std::sync::Mutex<BotState>>,
}

macro_rules! gen_set_props {
	($($prop:ident),*) => {
		$(
			impl LichessBot {
				/// create new lichess bot
				pub fn $prop(mut self, value: bool) -> LichessBot{
					self.$prop = value;

					self
				}
			}
		)*
	}
}

gen_set_props!(
	enable_classical,
	enable_rapid,
	disable_blitz,
	disable_bullet,
	enable_ultrabullet,
	enable_casual,
	disable_rated
);

/// lichess bot implementation
impl LichessBot {
	/// create new lichess bot
	pub fn new() -> LichessBot {
		let bot_name = std::env::var("RUST_BOT_NAME").unwrap();

		let max_book_depth:usize = env_or("RUST_BOT_BOOK_DEPTH", 20);

		let mut bot = LichessBot {
			lichess: Lichess::new(std::env::var("RUST_BOT_TOKEN").unwrap()),
			bot_name: bot_name.to_owned(),
			engine_name: std::env::var("RUST_BOT_ENGINE_NAME").ok(),
			uci_options: std::collections::HashMap::new(),
			enable_classical: false,
			enable_rapid: false,
			disable_blitz: false,
			disable_bullet: false,
			enable_ultrabullet: false,
			enable_casual: false,
			disable_rated: false,
			book: Book::new().me(bot_name.to_owned()),
			state: std::sync::Arc::new(std::sync::Mutex::new(BotState{current_fen: None}))
		}.max_book_depth(max_book_depth);

		bot.book.parse(env_string_or("RUST_BOT_BOOK_PGN", "book.pgn"));

		if log_enabled!(Level::Info) {
			info!("max book depth {}", bot.book.max_depth);
		}

		bot
	}

	/// set max book depth
	pub fn max_book_depth<T>(mut self, max_book_depth: T) -> LichessBot
	where T: core::fmt::Display {
		self.book = self.book.max_depth(max_book_depth);

		self
	}

	/// add uci option
	pub fn uci_opt<K, V>(mut self, key: K, value: V) -> LichessBot
	where K: core::fmt::Display, V: core::fmt::Display {
		let key = key.to_string();
		let value = value.to_string();

		self.uci_options.insert(key, value);

		self
	}

	/// play game
	async fn play_game(&mut self, game_id: String) -> Result<(), Box::<dyn std::error::Error>> {
		if log_enabled!(Level::Info) {
			info!("playing game {}", game_id);
		}

		let mut game_stream = self.lichess
			.stream_bot_game_state(&game_id)
			.await
			.unwrap();
		
		let mut bot_white = true;					

		let engine:Option<std::sync::Arc<uciengine::uciengine::UciEngine>> = match self.engine_name.to_owned() {
			Some(engine_name) => {
				if log_enabled!(Level::Debug) {
					debug!("created engine for playing game");
				}

				Some(UciEngine::new(engine_name))
			},
			_ => {
				if log_enabled!(Level::Debug) {
					debug!("no engine available for playing game");
				}

				None
			}
		};

		let mut ponder:Option<String> = None;
		
		while let Some(game_event) = game_stream.try_next().await? {
			if log_enabled!(Level::Debug) {
				debug!("game event {:?}", game_event);
			}

			let white:String;
			let black:String;			
			
			let state_opt = match game_event {
				BoardState::GameFull ( game_full ) => {
					if log_enabled!(Level::Debug) {
						debug!("game full {:?}", game_full);
					}
					
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
					
					if log_enabled!(Level::Info) {
						info!("**************\n{} - {} ( bot playing white {} )\n**************", white, black, bot_white);
					}
					
					Some(game_full.state)
				},
				BoardState::GameState ( game_state ) => {
					Some(game_state)
				},
				_ => {
					if log_enabled!(Level::Debug) {
						debug!("undhandled game event {:?}", game_event);
					}

					None
				}
			};

			if state_opt.is_some() {
				let state = state_opt.unwrap();

				if log_enabled!(Level::Debug) {
					debug!("game state {:?}", state);
				}

				let (fen, epd) = make_uci_moves(state.moves.as_str())?;

				let self_state_clone = self.state.clone();
				let mut self_state = self_state_clone.lock().unwrap();
    			self_state.current_fen = Some(fen.to_owned());
    			drop(self_state);

				if log_enabled!(Level::Debug) {
					debug!("fen of current position {}", fen);
				}

				let setup: Fen = fen.parse()?;
				let pos: Chess = setup.position(shakmaty::CastlingMode::Standard)?;

				let legals = pos.legals();

				if legals.len() > 0 {
					let rand_move = legals.choose(&mut rand::thread_rng()).unwrap();

					let rand_uci = Uci::from_standard(&rand_move).to_string();

					if log_enabled!(Level::Debug) {
						debug!("rand uci {}", rand_uci);
					}

					let turn = setup.turn;

					if log_enabled!(Level::Debug) {
						debug!("turn {:?}", turn);
					}

					let bot_turn = ( ( turn == Color::White ) && bot_white ) || ( ( turn == Color::Black ) && !bot_white );

					if log_enabled!(Level::Debug) {
						debug!("bot turn {}", bot_turn);
					}

					if bot_turn {
						let mut bestmove = rand_uci;

						let pos = self.book.positions.get(&epd);

						let mut has_book_move = false;

						if let Some(pos) = pos {
							let mixed:usize = env_or("RUST_BOT_MIXED", 90);

							if log_enabled!(Level::Info) {
								info!("searching for random move by mixed {}", mixed);
							}

							if let Some(m) = pos.get_random_mixed(mixed) {
								bestmove = m.uci.to_owned();

								has_book_move = true;

								if log_enabled!(Level::Info) {
									info!("book move found {}", bestmove);
								}
							}
						}

						let id = game_id.to_owned();

						if engine.is_some() && (!has_book_move) {
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

							let mut ponderhit = false;
							let mut pondermiss = false;

							if ( state.moves.len() > 0 ) && ( ponder.is_some() ) {
								// check ponder
								let mut moves_array:Vec<&str> = state.moves.split(" ").collect();

								let last_uci = moves_array.pop().unwrap();

								if log_enabled!(Level::Info) {
									info!("incoming uci {}", last_uci);
								}

								ponderhit = match ponder {
									Some(ref uci) => {
										if log_enabled!(Level::Info) {
											info!("expected uci {}", uci);
										}

										last_uci == uci
									},
									_ => false
								};

								pondermiss = !ponderhit;
							}

							let elapsed:u128;

							let go_result = match ponderhit || pondermiss {
								true => {
									if log_enabled!(Level::Info) {
										info!("ponderhit {} pondermiss {}", ponderhit, pondermiss);
									}

									if pondermiss {
										if log_enabled!(Level::Info) {
											info!("pondermiss, stopping engine");
										}

										let start = std::time::Instant::now();

										let _ = engine.clone().unwrap().go(GoJob::new().pondermiss()).recv().await;

										if log_enabled!(Level::Info) {
											info!("engine start from scratch thinking on {:?}", go_job);
										}

										let result = engine.clone().unwrap().go(go_job).recv().await;

										elapsed = start.elapsed().as_millis();

										result
									}else{
										if log_enabled!(Level::Info) {
											info!("ponderhit, waiting for result");
										}

										let start = std::time::Instant::now();

										let result = engine.clone().unwrap().go(GoJob::new().ponderhit()).recv().await;

										elapsed = start.elapsed().as_millis();

										result
									}
								},
								_ => {
									if log_enabled!(Level::Info) {
										info!("engine start thinking on {:?}", go_job);
									}

									let mut go_job = go_job;

									for (key, value) in &self.uci_options {
										if log_enabled!(Level::Info) {
											info!("adding uci option {} = {}", key, value);
										}

										go_job = go_job.uci_opt(key, value);
									}

									if log_enabled!(Level::Debug) {
										debug!("mounted go job {:?}", go_job);
									}

									let start = std::time::Instant::now();

									let result = engine.clone().unwrap().go(go_job).recv().await;

									elapsed = start.elapsed().as_millis();

									result
								}
							};

							if log_enabled!(Level::Debug) {
								debug!("thinking took {} ms , result {:?}", elapsed, go_result);
							}

							let mut eff_wtime = state.wtime as i32;
							let mut eff_btime = state.btime as i32;

							if bot_white {
								eff_wtime -= elapsed as i32;

								if eff_wtime < 0 {
									eff_wtime = 100;
								}

								if log_enabled!(Level::Info) {
									info!("changing wtime from {} to {}", state.wtime, eff_wtime);
								}
							} else {
								eff_btime -= elapsed as i32;

								if eff_btime < 0 {
									eff_btime = 100;
								}

								if log_enabled!(Level::Info) {
									info!("changing btime from {} to {}", state.btime, eff_btime);
								}
							}

							if let Some(go_result) = go_result {
								if let Some(bm) = go_result.bestmove {
									bestmove = bm;
								}

								if log_enabled!(Level::Debug) {
									debug!("ponder before {:?}", ponder);
								}

								ponder = go_result.ponder;

								if log_enabled!(Level::Info) {
									info!("set ponder to {:?}", ponder);
								}

								if let Some(ref uci) = ponder {
									let new_moves = match state.moves.as_str() {
										"" => format!("{} {}", bestmove, uci),
										_ => format!("{} {} {}", state.moves, bestmove, uci)
									};

									if log_enabled!(Level::Info) {
										info!("start pondering on {}", new_moves);
									}

									let go_job_ponder = GoJob::new()
											.uci_opt("UCI_Variant", "chess")
											.pos_startpos()
											.pos_moves(new_moves)
											.ponder()
											.tc(Timecontrol{
												wtime: eff_wtime as usize,
												winc: state.winc as usize,
												btime: eff_btime as usize,
												binc: state.binc as usize
											})
										;

									let _ = engine.clone().unwrap().go(go_job_ponder);
								}
							}
						} else {
							if log_enabled!(Level::Info) {
								info!("no engine available, making random move");
							}
						}

						if log_enabled!(Level::Info) {
							info!("making move {}", bestmove);
						}

						let result = self.lichess.make_a_bot_move(id.as_str(), bestmove.as_str(), false).await;

						if log_enabled!(Level::Debug) {
							debug!("make move result {:?}", result);
						}
					}
				} else {
					if log_enabled!(Level::Info) {
						info!("position has no legal move");
					}
				}
			}
		}

		if engine.is_some() {
			// stop engine before quitting
			let _ = engine.clone().unwrap().go(GoJob::new().custom("stop"));

			// quit engine
			engine.unwrap().quit();
		}

		Ok(())
	}

	/// process event stream event
	async fn process_event_stream_event(&mut self, event: Event) -> Result<(), Box::<dyn std::error::Error>> {
		if log_enabled!(Level::Debug) {
			debug!("event {:?}", event);
		}

		match event {
			Event::Challenge { challenge } => {
				if log_enabled!(Level::Info) {
					info!("incoming challenge {:?}", challenge.id);
				}
				
				if challenge.variant.key == "standard" {
					if challenge.speed == "correspondence" {
						if log_enabled!(Level::Info) {
							info!("rejecting challenge, correspondence");
						}
					} else {
						let mut challenge_ok = true;

						let mut decline_reasons:Vec<String> = vec!();

						if challenge.speed == "classical" {
							if !self.enable_classical {
								challenge_ok = false;

								decline_reasons.push(format!("{}", "wrong speed ( classical )"));
							}
						}

						if challenge.speed == "rapid" {
							if !self.enable_rapid {
								challenge_ok = false;

								decline_reasons.push(format!("{}", "wrong speed ( rapid )"));
							}
						}

						if challenge.speed == "blitz" {
							if self.disable_blitz {
								challenge_ok = false;

								decline_reasons.push(format!("{}", "wrong speed ( blitz )"));
							}
						}

						if challenge.speed == "bullet" {
							if self.disable_bullet {
								challenge_ok = false;

								decline_reasons.push(format!("{}", "wrong speed ( bullet )"));
							}
						}

						if challenge.speed == "ultrabullet" {
							if !self.enable_ultrabullet {
								challenge_ok = false;

								decline_reasons.push(format!("{}", "wrong speed ( ultrabullet )"));
							}
						}

						if challenge.rated {
							if self.disable_rated {
								challenge_ok = false;

								decline_reasons.push(format!("{}", "wrong mode ( rated )"));
							}
						}

						if !challenge.rated {
							if !self.enable_casual {
								challenge_ok = false;

								decline_reasons.push(format!("{}", "wrong mode ( casual )"));
							}
						}

						if challenge_ok {
							if log_enabled!(Level::Info) {
								info!("accepting challenge, response {:?}",
									self.lichess.challenge_accept(&challenge.id).await);																							
							}
						} else {
							if log_enabled!(Level::Info) {
								info!("declining challenge {:?}", decline_reasons);									
							}
						}		
					}
				} else {
					if log_enabled!(Level::Info) {
						info!("rejecting challenge, wrong variant {}", challenge.variant.key);
					}
				}
			},
			Event::GameStart { game } => {
				let game_id = format!("{}", game.id);

				if log_enabled!(Level::Info) {
					info!("game started {}", game_id);
				}

				let result = self.play_game(game_id).await;

				if log_enabled!(Level::Info) {
					info!("playing game finished with result {:?}", result);
				}
			}
			_ => {
				if log_enabled!(Level::Debug) {
					debug!("unhandled event {:?}", event)
				}
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
