use futures_util::TryStreamExt;
use licoricedev::client::{Lichess};
use licoricedev::models::board::{Event};

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
							self.lichess.challenge_accept(&challenge.id).await.unwrap());															}
				} else {
					println!("rejecting challenge, wrong variant {}", challenge.variant.key);
				}
			},
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
