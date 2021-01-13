//!
//! # Examples
//!
//!
//!```
//!extern crate env_logger;
//!
//!use dotenv::dotenv;
//!
//!use lichessbot::lichessbot::*;
//!
//!#[tokio::main]
//!async fn main() -> Result<(), Box<dyn std::error::Error>>{
//!	dotenv().ok();
//!	env_logger::init();
//!
//!	let mut bot = LichessBot::new();
//!
//!	let _ = bot.stream().await;
//!
//!	Ok(())
//!}
//!
//!```


// lib
pub mod lichessbot;