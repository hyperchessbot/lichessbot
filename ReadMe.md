# lichessbot

[![documentation](https://docs.rs/lichessbot/badge.svg)](https://docs.rs/lichessbot) [![Crates.io](https://img.shields.io/crates/v/lichessbot.svg)](https://crates.io/crates/lichessbot) [![Crates.io (recent)](https://img.shields.io/crates/dr/lichessbot)](https://crates.io/crates/lichessbot)

Lichess bot. Under construction.

# Usage

```rust
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
    	println!("event {:?}", event)
    }

	Ok(())
}

```

# Logging

```bash
export RUST_LOG=info
# or 
export RUST_LOG=debug
```
