extern crate env_logger;

use dotenv::dotenv;

use lichessbot::lichessbot::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();
	env_logger::init();

	let mut bot = LichessBot::new()
		.uci_opt("Move Overhead", 500)
		.uci_opt("Threads", 4)
		.uci_opt("Hash", 128)
		.uci_opt("Contempt", -25)
		.enable_classical(false)
		.enable_rapid(false)
		.disable_blitz(false)
		.enable_ultrabullet(false)
		.enable_casual(true)
		.disable_rated(false)
	;

	bot.stream().await
}
