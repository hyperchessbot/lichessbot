use dotenv::dotenv;

use lichessbot::lichessbot::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();

	let mut bot = LichessBot::new();

	bot.stream().await
}
