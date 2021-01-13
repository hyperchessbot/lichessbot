use futures_util::TryStreamExt;
use licoricedev::client::{Lichess};
use licoricedev::models::board::{Event};

pub struct LichessBot {
}

impl LichessBot {
	pub fn new() -> LichessBot {
		LichessBot {

		}
	}

	pub fn process_event_stream_event(&mut self, event: Event){
		println!("event {:?}", event)
	}

	pub async fn stream(&mut self) -> Result<(), Box::<dyn std::error::Error>> {
		let lichess = Lichess::new(std::env::var("RUST_BOT_TOKEN").unwrap());
	
		let mut event_stream = lichess
			.stream_incoming_events()
			.await
			.unwrap();

		while let Some(event) = event_stream.try_next().await? {			
	    	self.process_event_stream_event(event);
	    }

	    Ok(())
	}
}
