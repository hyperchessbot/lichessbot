# lichessbot

[![documentation](https://docs.rs/lichessbot/badge.svg)](https://docs.rs/lichessbot) [![Crates.io](https://img.shields.io/crates/v/lichessbot.svg)](https://crates.io/crates/lichessbot) [![Crates.io (recent)](https://img.shields.io/crates/dr/lichessbot)](https://crates.io/crates/lichessbot)

Lichess bot. Under construction.

# Usage

```rust
extern crate env_logger;

use dotenv::dotenv;

use lichessbot::lichessbot::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();
	env_logger::init();

	let mut bot = LichessBot::new();

	let _ = bot.stream().await;

	Ok(())
}

```

# Logging

```bash
export RUST_LOG=info
# or 
export RUST_LOG=debug
```
