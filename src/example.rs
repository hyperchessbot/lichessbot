extern crate env_logger;

use dotenv::dotenv;

use futures_util::TryStreamExt;
use licoricedev::client::{Lichess};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();
	env_logger::init();

	let lichess = Lichess::new(std::env::var("RUST_BOT_TOKEN").unwrap());
	
	let mut event_stream = lichess
		.stream_incoming_events()
		.await
		.unwrap();

	while let Some(event) = event_stream.try_next().await? {
    	let event = event;

    	println!("event {:?}", event)
    }

	Ok(())
}
